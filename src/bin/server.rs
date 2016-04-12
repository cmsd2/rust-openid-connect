extern crate openid_connect;

extern crate iron;
extern crate router;
extern crate logger;
extern crate urlencoded;
extern crate handlebars_iron;

#[macro_use] extern crate log;
extern crate env_logger;

use iron::prelude::*;
use iron::status;
use iron::method::Method;
use iron::{AfterMiddleware, Handler};
use router::Router;
use logger::Logger;
use logger::format::Format;
use urlencoded::{UrlEncodedQuery, UrlEncodedBody};
use handlebars_iron::{HandlebarsEngine, DirectorySource, Template};
use std::collections::HashMap;
use std::io::Read;

use openid_connect::{AuthorizeRequest};
use openid_connect::result::*;
use openid_connect::params::*;
use openid_connect::login::*;

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
    debug!("/authorize");
    let authorize_request = try!(parse_authorize_request(req));
    debug!("authorize: {:?}", authorize_request);
    let mut data = HashMap::<String,String>::new();
    data.insert("csrf_token".to_owned(), "foo".to_owned());
    
    //TODO validate subject claim
    Ok(Response::with((iron::status::Ok, Template::new("authorize/login.html", data))))
}

pub fn parse_login_request(req: &mut Request) -> Result<LoginRequest> {
    let mut s: String = "".to_owned();
//    let body = req.body;
    let _size = req.body.read_to_string(&mut s).unwrap();
    debug!("body: {}", s);
    
//    let hashmap = try!(req.get_ref::<UrlEncodedBody>());
    
    //TODO validate csrf_token
    //TODO check credentials
    
//    let login_request = try!(LoginRequest::from_params(hashmap));
    
//    Ok(login_request)
    Err(OpenIdConnectError::NotImplemented)
}

pub fn login_handler(req: &mut Request) -> IronResult<Response> {
    debug!("/login");
    match parse_login_request(req) {
        Ok(login_request) => {
            if req.method == Method::Post {
                debug!("try logging in {:?}", login_request);
            } else {
                debug!("render form {:?}", login_request);
            }
        },
        Err(err) => {
            debug!("render form with error {:?}", err);
        }
    }
    
    //TODO if post then try login
    
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

fn hello_handler(_: &mut Request) -> IronResult<Response> {
    let mut resp = Response::new();

    let mut data = HashMap::<String,String>::new();
    data.insert("msg".to_owned(), "Hello, World!".to_owned());
    resp.set_mut(Template::new("index", data)).set_mut(status::Ok);
    Ok(resp)
}

pub fn main() {
    env_logger::init().unwrap();
    let format = Format::new(FORMAT, vec![], vec![]);
    let (logger_before, logger_after) = Logger::new(Some(format.unwrap()));
    
    // html content type;
    // html error pages
    // urlencoded_form accept type?
    // form request forgery protection
    // TODO move the hbse out to be reused
    // TODO macro syntax to wrap several routes similarly
    fn web_handler<T>(route: T) -> Chain
    where T: Handler
    {
        let mut hbse = HandlebarsEngine::new();
        hbse.add(Box::new(DirectorySource::new("./templates/", ".hbs")));
        if let Err(r) = hbse.reload() {
            panic!("{:?}", r);
        }
  
        let mut chain = Chain::new(route);
        chain.link_after(hbse);
        chain
    }
    
    // json accept and content types
    // json error page
    // jwt validation
    fn api_handler<T>(route: T) -> Chain
    where T: Handler
    {
        let mut chain = Chain::new(route);
        chain
    }
    
    
    let mut router = Router::new();
//    router.get("/.well-known/)
    router.get("/authorize", web_handler(authorize_handler));
    router.get("/hello", web_handler(hello_handler));
    router.get("/login", web_handler(login_handler));
    router.post("/login", web_handler(login_handler));
    
    let mut chain = Chain::new(router);
    
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    chain.link_after(ErrorRenderer);
    
    Iron::new(chain).http("localhost:3000").unwrap();
}