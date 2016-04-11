extern crate openid_connect;

extern crate iron;
extern crate router;
extern crate logger;

#[macro_use] extern crate log;
extern crate env_logger;

use iron::prelude::*;
use iron::status;
use router::Router;
use logger::Logger;
use logger::format::Format;

use openid_connect::{AuthorizeRequest};
use openid_connect::result::*;

// without colours so it works on conhost terminals
static FORMAT: &'static str =
        "{method} {uri} -> {status} ({response-time} ms)";

pub fn parse_authorize_request(req: &Request) -> Result<AuthorizeRequest> {
    Err(OpenIdConnectError::NotImplemented)
}

pub fn authorize_handler(req: &mut Request) -> IronResult<Response> {
    debug!("Ok");
    let _authorize_request = try!(parse_authorize_request(req));
    Ok(Response::with((iron::status::Ok, "Ok")))
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
    
    Iron::new(chain).http("localhost:3000").unwrap();
}