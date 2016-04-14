extern crate iron;
extern crate router;
extern crate urlencoded;
extern crate handlebars_iron;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate log;

pub mod result;
pub mod params;
pub mod urls;
pub mod routes;
pub mod authentication;
pub mod users;
pub mod config;
pub mod handlers;
pub mod validation;

use result::{Result, OpenIdConnectError};

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


#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
