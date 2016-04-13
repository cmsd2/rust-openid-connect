extern crate iron;
extern crate urlencoded;
extern crate handlebars_iron;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate log;

pub mod result;
pub mod params;
pub mod login;
pub mod authorize;
pub mod urls;

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


#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
