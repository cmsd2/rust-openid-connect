use std::collections::HashMap;
use result::{Result, OpenIdConnectError};
use params::*;

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
