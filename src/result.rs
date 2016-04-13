use std::result;
use std::io;

use iron::prelude::*;
use iron::status;
use urlencoded;
use params;
use authentication;

quick_error! {
    #[derive(Debug)]
    pub enum OpenIdConnectError {
        IoError(err: io::Error) {
            from()
            description("io error")
            display("I/O error: {}", err)
            cause(err)
        }
      
        UnknownResponseType(response_type: Box<String>) {
            description("unknown response_type")
            display("Unknown response_type: {}", response_type.as_ref())
        }
        
        ScopeNotFound(scope: Box<String>) {
            description("scope not found")
            display("Scope not found: {}", scope)
        }
        
        NotImplemented {
            description("not implemented")
            display("Not implemented")
        }
        
        UrlDecodingError(err: urlencoded::UrlDecodingError) {
            from()
            description("url decoding error")
            display("Url decoding error: {}", err)
            cause(err)
        }
        
        ParamError(err: params::ParamError) {
            from()
            description("param error")
            display("Param error: {}", err)
            cause(err)
        }
        
        UrlParseError(description: String) {
            description("url parse error")
            display("Error parsing url: {}", description)
        }
        
        UserAlreadyExists {
            description("user already exists")
            display("User already exists")
        }
        
        UserNotFound {
            description("user not found")
            display("User not found")
        }
    }
}

pub fn error_status_code(oic_err: &OpenIdConnectError) -> status::Status {
    match *oic_err {
        OpenIdConnectError::UrlDecodingError(ref _err) => status::BadRequest,
        OpenIdConnectError::UnknownResponseType(ref _response_type) => status::BadRequest,
        OpenIdConnectError::ParamError(ref _response_type) => status::BadRequest,
        OpenIdConnectError::ScopeNotFound(ref _scope) => status::BadRequest,
        _ => status::InternalServerError
    }
}

impl From<OpenIdConnectError> for IronError {
    fn from(err: OpenIdConnectError) -> IronError {
        let status_code = error_status_code(&err);
        
        IronError::new(err, status_code)
    }
}

pub type Result<T> = result::Result<T,OpenIdConnectError>;