use std::io::Read;
use std::collections::HashMap;

use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;
use iron_sessionstorage::traits::*;
use urlencoded::*;
use serde_json::value;

use jsonwebtoken::validation::*;
use rbvt::params::*;
use result::{Result, OpenIdConnectError};
use urls::*;
use config::Config;
use view::View;
use back::*;

#[derive(Clone, Debug)]
pub struct LoginRequest {
    username: String,
    password: String,
    csrf_token: String,
}

#[derive(Clone, Debug)]
pub struct LoginRequestBuilder {
    username: Option<String>,
    password: Option<String>,
    csrf_token: Option<String>,
    
    validation_state: ValidationState,
}

impl LoginRequestBuilder {
    pub fn new() -> LoginRequestBuilder {
        LoginRequestBuilder {
            username: None,
            password: None,
            csrf_token: None,
            
            validation_state: ValidationState::new(),
        }
    }
    
    pub fn build(self) -> Result<LoginRequest> {
        if self.validation_state.valid {
            Ok(LoginRequest {
                username: self.username.unwrap(),
                password: self.password.unwrap(),
                csrf_token: self.csrf_token.unwrap(),
            })
        } else {
            Err(OpenIdConnectError::from(ValidationError::ValidationError(self.validation_state)))
        }
    }
    
    pub fn load_params(&mut self, hashmap: &HashMap<String, Vec<String>>) -> Result<bool> {
        if let Some(username) = try!(multimap_get_maybe_one(hashmap, "username")) {
            self.username = Some(username.to_owned());
        } else {
            self.validation_state.reject("username", ValidationError::MissingRequiredValue("username".to_owned()));
        }
        
        if let Some(password) = try!(multimap_get_maybe_one(hashmap, "password")) {
            self.password = Some(password.to_owned());
        } else {
            self.validation_state.reject("password", ValidationError::MissingRequiredValue("password".to_owned()));
        }
        
        if let Some(csrf_token) = try!(multimap_get_maybe_one(hashmap, "csrf_token")) {
            self.csrf_token = Some(csrf_token.to_owned());
        } else {
            self.validation_state.reject("csrf_token", ValidationError::MissingRequiredValue("csrf_token".to_owned()));
        }
        
        Ok(self.validation_state.valid)
    }
    
    pub fn build_from_params(params: &HashMap<String, Vec<String>>) -> Result<LoginRequest> {
        let mut builder = LoginRequestBuilder::new();
        
        try!(builder.load_params(params));
        
        builder.build()
    }
}

pub fn parse_login_request(req: &mut Request) -> Result<LoginRequest> {
    let mut s: String = "".to_owned();
//    let body = req.body;
    let _size = req.body.read_to_string(&mut s).unwrap();
    debug!("body: {}", s);
    
//    let hashmap = try!(req.get_ref::<UrlEncodedBody>());
    
    //TODO validate csrf_token
    //TODO check credentials
    
//    let login_request = try!(LoginRequest::from_params(hashmap));
    
//    Ok(login_request)
    Err(OpenIdConnectError::NotImplemented)
}

pub fn login_path() -> &'static str {
    "/login"
}

/// called by user agent, probably redirected from authorize
/// login with cookie if possible
/// if not logged in or reprompting for credentials, render login form
/// otherwise redirect to caller
/// on error set flash error and render login form
pub fn login_get_handler(req: &mut Request) -> IronResult<Response> {
    let mut view = try!(View::new_for_session("login.html", req));
    
    match req.get_ref::<UrlEncodedQuery>() {
        Ok(params) => {
            view.data.insert("return".to_owned(), 
                try!(value::to_value(&try!(multimap_get_maybe_one(params, "return").map_err(OpenIdConnectError::from)))
                    .map_err(OpenIdConnectError::from))
            );
            
            match LoginRequestBuilder::build_from_params(&params) {
                Ok(login_request) => {
                    // handlebars escapes these for us
                    view.data.insert("username".to_owned(), try!(value::to_value(&login_request.username).map_err(OpenIdConnectError::from)));
                    view.data.insert("password".to_owned(), try!(value::to_value(&login_request.password).map_err(OpenIdConnectError::from)));
                }
                Err(err) => {
                    debug!("error parsing login form: {:?}", err);
                }
            }
            debug!("parsed query params: {:?}", params);
        },
        Err(err) => {
            debug!("error parsing query params: {:?}", err);
        }
    }
    
    Ok(Response::with((status::Ok, try!(view.template().map_err(OpenIdConnectError::from)))))
}

/// called by user agent form post
/// login with credentials if possible
/// if not logged in, render login form with flash error
/// otherwise redirect to caller
/// on error, set flash error and render login form
pub fn login_post_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    
    let login_url = try!(relative_url(req, "/connect/login", None));
    let home_url = try!(relative_url(req, "/", None));
    
    let params = try!(match req.get::<UrlEncodedBody>() {
        Ok(params) => Ok(params),
        Err(UrlDecodingError::EmptyQuery) => Ok(HashMap::new()),
        Err(e) => Err(e)
    }.map_err(OpenIdConnectError::from));
    
    // TODO handle cancel button
    
    match config.session_controller.login_with_credentials(req) {
        Ok(login) => {
            if let Some(session) = login.session {
                try!(req.session().set(session));

                Ok(Response::with((status::Found, Redirect(try!(redirect_back_url(req, &params)).unwrap_or(home_url)))))
            } else {
                Err(IronError::from(OpenIdConnectError::NoSessionLoaded))
            }
        },
        Err(err) => {
            debug!("error logging in: {:?}", err);
            Ok(Response::with((status::Found, Redirect(try!(redirect_back_url(req, &params)).unwrap_or(login_url)))))
        }
    }
}
