use std::result;
use std::io;

quick_error! {
    #[derive(Debug)]
    pub enum OpenIdConnectError {
        IoError(err: io::Error) {
            from()
            description("io error")
            display("I/O error: {}", err)
            cause(err)
        }
      
        UnknownResponseTypeError(response_type: Box<String>) {
            description("unknown response_type")
            display("Unknown response_type: {}", response_type.as_ref())
        }
    }
}
    
pub type Result<T> = result::Result<T,OpenIdConnectError>;