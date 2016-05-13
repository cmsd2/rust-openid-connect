use std::result;
use std::io;

use iron::prelude::*;
use iron::status;
use urlencoded;
use bodyparser;
use serde_json;
use rbvt::params;
use rbvt;
use url;
use persistent;
use jsonwebtoken::result::*;

quick_error! {
    #[derive(Debug)]
    pub enum OpenIdConnectError {
        IoError(err: io::Error) {
            from()
            description("io error")
            display("I/O error: {}", err)
            cause(err)
        }
      
        UnknownResponseMode(rm: Box<String>) {
            description("unknown response_mode")
            display("Unknown response_mode: {}", rm.as_ref())
        }
        
        UnknownResponseType(response_type: Box<String>) {
            description("unknown response_type")
            display("Unknown response_type: {}", response_type.as_ref())
        }
        
        UnknownGrantType(grant_type: Box<String>) {
            description("unknown grant_type")
            display("Unknown grant_type: {}", grant_type.as_ref())
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
        
        UrlError(description: String) {
            description("url parse error")
            display("Error parsing url: {}", description)
        }
        
        UrlParseError(err: url::ParseError) {
            from()
            description("url parse error")
            display("Error parsing url: {}", err)
            cause(err)
        }
        
        UserAlreadyExists {
            description("user already exists")
            display("User already exists")
        }
        
        UserNotFound {
            description("user not found")
            display("User not found")
        }
        
        ClientApplicationAlreadyExists {
            description("application already exists")
            display("Application already exists")
        }
        
        ClientApplicationNotFound {
            description("application not found")
            display("Application not found")
        }
        
        InvalidRedirectUri {
            description("redirect uri is not recognised")
            display("Redirect uri is not recognised")
        }
        
        ValidationError(err: rbvt::result::ValidationError) {
            from()
            description("validation error")
            display("Validation error: {}", err)
            cause(err)
        }
        
       /* JsonEncoderError(err: EncoderError) {
            from()
            description("error encoding json")
            display("Error encoding json: {}", err)
            cause(err)
        }
        
        JsonDecoderError(err: DecoderError) {
            from()
            description("error decoding json")
            display("Error decoding json: {}", err)
            cause(err)
        }*/
        
        EmptyPostBody {
            description("empty post body")
            display("Empty post body")
        }
        
        JsonError(err: serde_json::Error) {
            from()
            description("json error")
            display("Json error: {}", err)
            cause(err)
        }
        
        PostBodyParseError(err: bodyparser::BodyError) {
            from()
            description("error parsing post body")
            display("Error parsing post body: {}", err)
            cause(err)
        }
        
        PersistentError(err: persistent::PersistentError) {
            from()
            description("persistence error")
            display("Persistence error: {}", err)
            cause(err)
        }
        
        InvalidUsernameOrPassword {
            description("invalid username or password")
            display("Invalid username or password")
        }
        
        NoSessionLoaded {
            description("server didn't load user session")
            display("server didn't load user session")
        }
        
        JwtError(e: JwtError) {
            from()
            description("jwt error")
            display("jwt error: {}", e)
            cause(e)
        }
        
        RoutingError(msg: String) {
            description("routing error")
            display("routing error: {}", msg)
        }
        
        GrantNotFound {
            description("grant not found")
            display("Grant not found")
        }
        
        ResponseModeUnavailable {
            description("the chosen response mode is unavailable for this authorize request")
            display("the chosen response mode is unavailable for this authorize request")
        }
    }
}

pub fn error_status_code(oic_err: &OpenIdConnectError) -> status::Status {
    match *oic_err {
        OpenIdConnectError::UrlDecodingError(ref _err) => status::BadRequest,
        OpenIdConnectError::UnknownResponseType(ref _response_type) => status::BadRequest,
        OpenIdConnectError::ParamError(ref _response_type) => status::BadRequest,
        OpenIdConnectError::ScopeNotFound(ref _scope) => status::BadRequest,
        OpenIdConnectError::JsonError(ref _err) => status::BadRequest,
        OpenIdConnectError::EmptyPostBody => status::BadRequest,
        OpenIdConnectError::ValidationError(ref _err) => status::BadRequest,
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