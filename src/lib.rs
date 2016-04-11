extern crate iron;
extern crate urlencoded;
#[macro_use] extern crate quick_error;

pub mod result;
pub mod params;

use std::collections::HashMap;
use result::{Result, OpenIdConnectError};
use params::*;

#[derive(Copy, Clone, Debug)]
pub enum ResponseType {
    Code,
}

impl ResponseType {
    pub fn from_str(s: &str) -> Result<ResponseType> {
        match s {
            "code" => Ok(ResponseType::Code),
            _ => Err(OpenIdConnectError::UnknownResponseType(Box::new(s.to_owned())))
        }
    }
}

#[derive(Clone, Debug)]
pub struct AuthorizeRequest {
    response_type: ResponseType,
    scopes: Vec<String>,
    client_id: String,
    state: String,
    // nonce: String, // ?
    redirect_uri: String, // or url type?
}

impl AuthorizeRequest {
    pub fn from_params(hashmap: &HashMap<String, Vec<String>>) -> Result<AuthorizeRequest> {
        let response_type = try!(multimap_get_one(hashmap, "response_type"));
        let scopes = try!(multimap_get(hashmap, "scope"));
        let client_id = try!(multimap_get_one(hashmap, "client_id"));
        let state = try!(multimap_get_one(hashmap, "state"));
        let redirect_uri = try!(multimap_get_one(hashmap, "redirect_uri"));
    
        Ok(AuthorizeRequest {
            response_type: try!(ResponseType::from_str(response_type)),
            scopes: scopes.clone(),
            client_id: client_id.to_owned(),
            state: state.to_owned(),
            redirect_uri: redirect_uri.to_owned(),
        })
    }
    
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().find(|s| *s == scope).is_some()
    }
}
    
#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
