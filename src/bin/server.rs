extern crate openid_connect;

extern crate iron;
extern crate router;
extern crate logger;
extern crate urlencoded;

#[macro_use] extern crate log;
extern crate env_logger;

use iron::prelude::*;
use iron::status;
use iron::{AfterMiddleware};
use router::Router;
use logger::Logger;
use logger::format::Format;
use urlencoded::UrlEncodedQuery;

use openid_connect::{AuthorizeRequest};
use openid_connect::result::*;
use openid_connect::params::*;

// without colours so it works on conhost terminals
static FORMAT: &'static str =
        "{method} {uri} -> {status} ({response-time} ms)";
    
pub fn parse_authorize_request(req: &mut Request) -> Result<AuthorizeRequest> {
    let hashmap = try!(req.get_ref::<UrlEncodedQuery>());
    
    //TODO validate supplied oauth2 params
    
    let auth_req = try!(AuthorizeRequest::from_params(hashmap));
    let openid_scope = "openid";
    
    if !auth_req.has_scope(openid_scope) {
        Err(OpenIdConnectError::ScopeNotFound(Box::new(openid_scope.to_owned())))
    } else {
        Ok(auth_req)
    }
}

pub fn authorize_handler(req: &mut Request) -> IronResult<Response> {
    debug!("Ok");
    let authorize_request = try!(parse_authorize_request(req));
    debug!("authorize: {:?}", authorize_request);
    Ok(Response::with((iron::status::Ok, "Ok")))
}

struct ErrorRenderer;

impl AfterMiddleware for ErrorRenderer {
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
        debug!("{:?} caught in ErrorRecover AfterMiddleware.", &err);
        
        let new_body = format!("{}", err);
        
        Ok(err.response.set(new_body))
    }
}

pub fn main() {
    env_logger::init().unwrap();
    let format = Format::new(FORMAT, vec![], vec![]);
    let (logger_before, logger_after) = Logger::new(Some(format.unwrap()));
    
    let mut router = Router::new();
//    router.get("/.well-known/)
    router.get("/authorize", authorize_handler);
    
    let mut chain = Chain::new(router);
    
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    chain.link_after(ErrorRenderer);
    
    Iron::new(chain).http("localhost:3000").unwrap();
}