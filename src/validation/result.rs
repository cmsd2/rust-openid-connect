use std::result;
use std::sync::Arc;
use std::io;
use std::error;

use super::params;
use super::state::ValidationState;

quick_error! {
    #[derive(Clone, Debug)]
    pub enum ValidationError {
        IoError(err: Arc<Box<io::Error>>) {
            from()
            description("io error")
            display("I/O error: {}", err)
            cause(err.get_ref().unwrap())
        }
        
        Error(err: Arc<Box<error::Error + Send + Sync>>) {
            from()
            description("error")
            display("Error: {}", err)
            //cause(err)
        }
        
        ApplicationError(err: String) {
            description("application error")
            display("Application error: {}", err)
        }
        
        ValidationError(state: ValidationState) {
            description("validation error")
            display("validation error: {:?}", state)
        }
        
        MissingRequiredValue(value_name: String) {
            description("missing required value")
            display("missing required value: {}", value_name)
        }
        
        InvalidValue(msg: String) {
            description("invalid value")
            display("invalid value: {}", msg)
        }
        
        ParamError(err: params::ParamError) {
            from()
            description("param error")
            display("param error: {}", err)
            cause(err)
        }
    }
}

pub type Result<T> = result::Result<T, ValidationError>;
