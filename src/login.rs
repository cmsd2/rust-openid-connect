use std::io::Read;
use std::collections::HashMap;
use std::result;

use iron::prelude::*;
use iron::status;
use iron::Url;
use iron::method::Method;
use iron::modifiers::Redirect;
use urlencoded::{UrlEncodedBody, UrlEncodedQuery};
use handlebars_iron::Template;

use result::{Result, OpenIdConnectError};
use params::*;
use urls::*;

#[derive(Clone, Debug)]
pub struct LoginRequest {
    username: String,
    password: String,
    csrf_token: String,
}

impl LoginRequest {
    pub fn from_params(hashmap: &HashMap<String, Vec<String>>) -> Result<LoginRequest> {
        let username = try!(multimap_get_one(hashmap, "username"));
        let password = try!(multimap_get_one(hashmap, "password"));
        let csrf_token = try!(multimap_get_one(hashmap, "csrf_token"));
        
        Ok(LoginRequest {
            username: username.to_owned(),
            password: password.to_owned(),
            csrf_token: csrf_token.to_owned(),
        })
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

pub fn login_get_handler(req: &mut Request) -> IronResult<Response> {
    let mut username = "".to_owned();
    let mut password = "".to_owned();
    
    match req.get_ref::<UrlEncodedQuery>() {
        Ok(params) => {
            debug!("parsed query params: {:?}", params);
        },
        Err(err) => {
            debug!("error parsing query params: {:?}", err);
        }
    }

    let mut data = HashMap::<String,String>::new();
    // TODO these must be escaped to avoid cross-site-scripting
    data.insert("username".to_owned(), username);
    data.insert("password".to_owned(), password);
    
    Ok(Response::with((status::Ok, Template::new("login.html", data))))
}

pub fn login_post_handler(req: &mut Request) -> IronResult<Response> {
    let login_url = try!(relative_url(req, "/login"));
    
    match req.get_ref::<UrlEncodedBody>() {
        Ok(params) => {
            // TODO validate csrf
            // TODO validate credentials
            // TODO create session and set cookie
            Ok(Response::with((status::Ok, "Ok")))
        },
        Err(err) => {
            debug!("error parsing body: {:?}", err);
            Ok(Response::with((status::Found, Redirect(login_url))))
        }
    }
}