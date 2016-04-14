use std::io::Read;
use std::collections::HashMap;

use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;
use urlencoded::{UrlEncodedBody, UrlEncodedQuery};
use handlebars_iron::Template;

use result::{Result, OpenIdConnectError};
use params::*;
use urls::*;
use config::Config;
use users::*;
use validation::*;

pub fn user_from_form(params: &HashMap<String, Vec<String>>) -> Result<User> {
    let username = try!(multimap_get_maybe_one(params, "username")).map(|s| s.to_owned()).unwrap_or(String::new());
    let password = try!(multimap_get_maybe_one(params, "password")).map(|s| s.to_owned());
        
    Ok(User::new(username, password))
}

pub fn validate_user(user: &User) -> Result<ValidatorState> {
    let mut validator = ValidatorSchema::<User>::new();
    
    validator.rule(Box::new(|u: &User, s: &mut ValidatorState| {
        if u.username == "" {
            s.reject("username", "username must not be empty".to_owned());
        }
        Ok(())
    }));
    
    validator.rule(Box::new(|u: &User, s: &mut ValidatorState| {
        if u.password.as_ref().map(|s| &s[..]).unwrap_or("") == "" {
            s.reject("password", "password must not be empty".to_owned());
        }
        Ok(())
    }));
    
    try!(validator.validate(user));
    
    debug!("user validation: {:?}", validator.state);
    
    Ok(validator.state)
}

pub fn load_register_form(params: &HashMap<String, Vec<String>>) -> Result<HashMap<String, String>> {
    let mut data = HashMap::<String, String>::new();
    
    match user_from_form(params) {
        Ok(user) => {
            let validation = try!(validate_user(&user));
            //TODO save validation results to hashmap for rendering
    
            data.insert("username".to_owned(), user.username.clone());
            data.insert("password".to_owned(), user.password.unwrap_or(String::new()).clone());
        },
        Err(err) => {
            debug!("error reading user fields from form: {:?}", err);
            //TODO render appropriate error message
        }
    }
    
    Ok(data)
}

pub fn new_register_form() -> HashMap<String, String> {
    let mut data = HashMap::new();
    
    data.insert("username".to_owned(), String::new());
    data.insert("password".to_owned(), String::new());
    
    data
}

pub fn register_get_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
    let mut username = "".to_owned();
    let mut password = "".to_owned();
    
    let mut data = match req.get_ref::<UrlEncodedQuery>() {
        Ok(params) => {
            debug!("parsed query params: {:?}", params);
            
            try!(load_register_form(params))
        },
        Err(err) => {
            debug!("error parsing query params: {:?}", err);
            
            new_register_form()
        }
    };
    
    data.insert("_view".to_owned(), "register.html".to_owned());
    
    Ok(Response::with((status::Ok, Template::new("_layout.html", data))))
}

pub fn register_post_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
    let register_url = try!(relative_url(req, "/register"));
    
    match req.get_ref::<UrlEncodedBody>() {
        Ok(params) => {
            debug!("registering new user with creds {:?}", params);
            // TODO validate csrf
            // TODO validate credentials
            // TODO create session and set cookie
            
            match user_from_form(params) {
                Ok(user) => {
                    debug!("add user to repo: {:?}", user);
                
                    config.user_repo.add_user(user);
                },
                Err(err) => {
                    debug!("user validation errors: {:?}", err);
                }
            }
            
            Ok(Response::with((status::Ok, "Ok")))
        },
        Err(err) => {
            debug!("error parsing body: {:?}", err);
            Ok(Response::with((status::Found, Redirect(register_url))))
        }
    }
}