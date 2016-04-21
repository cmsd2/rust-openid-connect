extern crate iron;
extern crate router;
extern crate urlencoded;
extern crate handlebars_iron;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate log;
extern crate vlad;

pub mod result;
pub mod urls;
pub mod routes;
pub mod authentication;
pub mod users;
pub mod client_application;
pub mod config;
pub mod handlers;

use result::{Result, OpenIdConnectError};

/// Authorization Code flow: "code"
/// Implicit flow: "id_token" or "id_token token"
/// Hybrid flow: "code id_token" or "code token" or "code id_token token"
#[derive(Copy, Clone, Debug)]
pub struct ResponseType {
    pub code: bool,
    pub id_token: bool,
    pub token: bool,
}

impl ResponseType {
    pub fn new(code: bool, id_token: bool, token: bool) -> ResponseType {
        ResponseType {
            code: code,
            id_token: id_token,
            token: token,
        }
    }
    
    pub fn from_str(s: &str) -> Result<ResponseType> {
        match s {
            "code" => Ok(ResponseType::new(true, false, false)),
            _ => Err(OpenIdConnectError::UnknownResponseType(Box::new(s.to_owned())))
        }
    }
}


#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
