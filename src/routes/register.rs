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
use view_models::User;
use users;

pub fn get_form_value<'a>(key: &str, params: &'a HashMap<String, Vec<String>>, flash: &mut Vec<String>) -> Option<&'a str> {
    let result = multimap_get_maybe_one(params, key);
    
    match result {
        Err(err) => {
            flash.push(format!("invalid {}: {}", key, err));
            None
        },
        Ok(s) => s
    }
}

pub fn user_from_form(params: &HashMap<String, Vec<String>>) -> User {
    let mut user = User::new();
    
    if let Some(username) = get_form_value("username", params, flash) {
        user.username = Some(username.to_owned());
    }
    
    if let Some(password) = get_form_value("password", params, flash) {
        user.password = Some(password.to_owned());
    }
    
    user
}

pub fn validate_user(user: &User, flash: &mut Vec<String>) -> Result<ValidatedUser> {
    let mut validating_user: User;
    let mut bool is_valid = false;
    
    if let Some(username) = get_form_value("username", params, flash) {
        user.username = Some(username.to_owned());
    }
    
    if let Some(password) = get_form_value("password", params, flash) {
        user.password = Some(password.to_owned());
    }
    
    if !flash.is_empty() {
        //TODO return errors to user
        debug!("form validation errors: {:?}", flash);
        Ok(user)
    } else {
        Ok(user)
    }
}

pub fn load_register_form(params: &HashMap<String, Vec<String>>) -> HashMap<String, String> {
    let mut data = HashMap::<String, String>::new();
    let mut flash: Vec<String> = vec![];
    
    if let Ok(user) = user_from_form(params, &mut flash) {
        data.insert("username".to_owned(), user.username.unwrap_or(String::new()).clone());
        data.insert("password".to_owned(), user.password.unwrap_or(String::new()).clone());
    }
    
    //TODO feed validation back to user
    if !flash.is_empty() {
        debug!("flash: {:?}", flash);
    }
    
    data
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
            
            load_register_form(params)
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
            let mut errors = vec![];
            
            if let Ok(user) = user_from_form(params, &mut errors) {
                debug!("add user to repo: {:?}", user);
                
                config.user_repo.add_user(users::User::new(user.username, user.password));
            } else {
                debug!("user validation errors: {:?}", errors);
            }
            
            Ok(Response::with((status::Ok, "Ok")))
        },
        Err(err) => {
            debug!("error parsing body: {:?}", err);
            Ok(Response::with((status::Found, Redirect(register_url))))
        }
    }
}