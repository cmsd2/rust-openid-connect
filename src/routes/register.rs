use std::collections::HashMap;

use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;
use urlencoded::{UrlEncodedBody, UrlEncodedQuery};
use serde_json::value;

use jsonwebtoken::validation::*;
use rbvt::params::*;

use result::{Result, OpenIdConnectError};
use urls::*;
use config::Config;
use users::*;
use authentication::*;
use view::View;

#[derive(Clone, Debug)]
pub struct RegisterRequest {
    username: String,
    password: String
}

#[derive(Clone, Debug)]
pub struct RegisterRequestBuilder {
    username: Option<String>,
    password: Option<String>,
}

impl RegisterRequestBuilder {
    pub fn new() -> RegisterRequestBuilder {
        RegisterRequestBuilder {
            username: None,
            password: None,
        }
    }
    
    pub fn build(self) -> Result<RegisterRequest> {
        Ok(RegisterRequest {
            username: self.username.unwrap(),
            password: self.password.unwrap(),
        })
    }
    
    pub fn validate(&self, state: &mut ValidationState) -> Result<bool> {
        if self.username.is_none() {
            state.reject("username", ValidationError::MissingRequiredValue("username".to_owned()));
        }
        
        if self.password.is_none() {
            state.reject("password", ValidationError::MissingRequiredValue("password".to_owned()));
        }
        
        Ok(state.valid)
    }
    
    pub fn load_params(&mut self, params: &HashMap<String, Vec<String>>) -> Result<()> {
        if let Some(username) = try!(multimap_get_maybe_one(params, "username")) {
            self.username = Some(username.to_owned());
        }
        
        if let Some(password) = try!(multimap_get_maybe_one(params, "password")) {
            self.password = Some(password.to_owned());
        }
        
        let mut validation_state = ValidationState::new();
        
        if ! try!(self.validate(&mut validation_state)) {
            Err(OpenIdConnectError::from(ValidationError::ValidationError(validation_state)))
        } else {
            Ok(())
        }
    }
    
    pub fn build_from_params(params: &HashMap<String, Vec<String>>) -> Result<RegisterRequest> {
        let mut builder = RegisterRequestBuilder::new();
        
        try!(builder.load_params(params));
        
        builder.build()
    }
    
    pub fn populate_view(&self, view: &mut View) -> Result<()> {
        view.data.insert("username".to_owned(), try!(value::to_value(&self.username)));
        view.data.insert("password".to_owned(), try!(value::to_value(&self.password)));
        Ok(())
    }
}

pub fn register_get_handler(req: &mut Request) -> IronResult<Response> {
    let mut view = try!(View::new_for_session("register.html", req));
    let mut register_form = RegisterRequestBuilder::new();
    
    match req.get_ref::<UrlEncodedQuery>() {
        Ok(params) => {
            debug!("parsed query params: {:?}", params);
            
            match RegisterRequestBuilder::build_from_params(params) {
                Ok(register_request) => {
                    //TODO escape values to protect against cross-site-scripting
                    register_form.username = Some(register_request.username);
                    register_form.password = Some(register_request.password);
                },
                Err(err) => {
                    debug!("invalid registration details: {:?}", err);
                }
            }
        },
        Err(err) => {
            debug!("error parsing query params: {:?}", err);
        }
    };
    
    register_form.populate_view(&mut view);
    
    Ok(Response::with((status::Ok, try!(view.template().map_err(OpenIdConnectError::from)))))
}

pub fn register_post_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    
    let register_url = try!(relative_url(req, "/register", None));
    let home_url =try!(relative_url(req, "/", None));
    
    match req.get::<UrlEncodedBody>() {
        Ok(params) => {
            debug!("registering new user with creds {:?}", params);
            // TODO validate csrf
            // TODO create session and set cookie
            // TODO multistep registration flow
            // TODO redirect to flow caller
            
            match RegisterRequestBuilder::build_from_params(&params) {
                Ok(register_request) => {
                    let user = User::new(new_user_id(), register_request.username, Some(register_request.password));
                    
                    debug!("add user to repo: {:?}", user);
                
                    // TODO render error as flash message
                    try!(config.user_repo.add_user(user));
                    
                    // TODO send email to user with confirmation token
                    
                    let login = try!(config.session_controller.login_with_credentials(req));
                    
                    Ok(Response::with((status::Found, Redirect(home_url))).set(login.cookie()))
                },
                Err(err) => {
                    debug!("user validation errors: {:?}", err);
                    
                    Ok(Response::with((status::Found, Redirect(register_url))))
                }
            }
        },
        Err(err) => {
            debug!("error parsing body: {:?}", err);
            Ok(Response::with((status::Found, Redirect(register_url))))
        }
    }
}