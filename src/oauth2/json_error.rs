use iron::prelude::*;
use iron::middleware::AfterMiddleware;
use iron::mime::Mime;
use serde_json;

use result::*;

pub struct JsonErrorRenderer;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorView {
    error: String,
}

impl AfterMiddleware for JsonErrorRenderer {
    /// if accept header contains */* or application/json
    /// then render error as json object
    /// otherwise pass error
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
        debug!("{:?} caught in ErrorRecover AfterMiddleware.", &err);
        
        let error_view = ErrorView { error: format!("{}", err) };
        
        // TODO render contents of error as json object instead of string
        let new_body = try!(serde_json::to_string(&error_view).map_err(OpenIdConnectError::from));
        
        let content_type = "application/json".parse::<Mime>().unwrap();
        
        Ok(err.response.set(new_body).set(content_type))
    }
}