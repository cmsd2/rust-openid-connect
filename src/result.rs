use std::result;
use std::io;

use iron::prelude::*;
use iron::status;

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
        
        NotImplemented {
            description("not implemented")
            display("Not implemented")
        }
    }
}

pub fn error_status_code(oic_err: &OpenIdConnectError) -> status::Status {
    match oic_err {
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