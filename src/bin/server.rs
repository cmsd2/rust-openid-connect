#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate openid_connect;
extern crate serde;
extern crate serde_json;
extern crate iron;
extern crate router;
extern crate logger;
extern crate urlencoded;
extern crate handlebars_iron;
extern crate staticfile;
extern crate mount;

#[macro_use] extern crate log;
extern crate env_logger;

use std::sync::Arc;
use std::path::Path;

use iron::prelude::*;
use iron::{AfterMiddleware};
use mount::Mount;
use staticfile::Static;
use router::Router;
use logger::Logger;
use logger::format::Format;
use handlebars_iron::{HandlebarsEngine, DirectorySource};

use openid_connect::routes::token::*;
use openid_connect::routes::login::*;
use openid_connect::routes::authorize::*;
use openid_connect::routes::home::*;
use openid_connect::routes::register::*;
use openid_connect::routes::application_api::*;
use openid_connect::routes::session::*;
use openid_connect::users::*;
use openid_connect::config::*;
use openid_connect::handlers::*;
use openid_connect::client_application::*;
use openid_connect::sessions;
use openid_connect::login;

// without colours so it works on conhost terminals
static FORMAT: &'static str =
        "{method} {uri} -> {status} ({response-time} ms)";
   
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
    
    let user_repo = Arc::new(Box::new(InMemoryUserRepo::new()) as Box<UserRepo>);
    user_repo.add_user(User::new("1".to_owned(), "admin".to_owned(), Some("admin".to_owned()))).unwrap();
    
    let application_repo = Arc::new(Box::new(InMemoryClientApplicationRepo::new()) as Box<ClientApplicationRepo>);
    
    let cookie_signing_key = b"My secret key"[..].to_owned();
    
    let sessions = Arc::new(Box::new(sessions::InMemorySessions::new(user_repo.clone())) as Box<sessions::Sessions>);
    let login_manager = login::LoginManager::new(cookie_signing_key);
    let sessions_controller = sessions::SessionController::new(sessions, login_manager.clone());
    
    let config = Config::new(user_repo, application_repo, sessions_controller);
    
    // html content type;
    // html error pages
    // urlencoded_form accept type?
    // form request forgery protection
    // TODO move the hbse out to be reused
    // TODO macro syntax to wrap several routes similarly
    fn web_handler<T>(config: &Config, route: T) -> Chain
    where T: MethodHandler<Config>
    {
        let mut hbse = HandlebarsEngine::new();
        hbse.add(Box::new(DirectorySource::new("./templates/", ".hbs")));
        if let Err(r) = hbse.reload() {
            panic!("{:?}", r);
        }
  
        let mut chain = Chain::new(bind(config.clone(), route));
        chain.link_after(hbse);
        chain
    }
    
    // json accept and content types
    // json error page
    // jwt validation
    fn api_handler<T>(config: &Config, route: T) -> Chain
    where T: MethodHandler<Config>
    {
        Chain::new(bind(config.clone(), route))
    }
    
    
    let mut router = Router::new();
//    router.get("/.well-known/)
    router.get("/authorize", web_handler(&config, authorize_handler));
    router.get("/", web_handler(&config, home_handler));
    router.get("/login", web_handler(&config, login_get_handler));
    router.post("/login", web_handler(&config, login_post_handler));
    router.get("/register", web_handler(&config, register_get_handler));
    router.post("/register", web_handler(&config, register_post_handler));
    
    router.post("/token", api_handler(&config, token_post_handler));
    
    let mut api_router = Router::new();
    api_router.get("/session", api_handler(&config, session_get_handler));
    api_router.post("/session", api_handler(&config, session_post_handler));
    api_router.delete("/session", api_handler(&config, session_delete_handler));
    
    api_router.get("/applications", api_handler(&config, applications_get_handler));
    api_router.post("/applications", api_handler(&config, applications_post_handler));
    api_router.put("/applications/:id", api_handler(&config, applications_put_handler));
    api_router.delete("/applications/:id", api_handler(&config, applications_delete_handler));
    
    let mut mount = Mount::new();
    mount.mount("/", router);
    mount.mount("/api", api_router);
    mount.mount("/js", Static::new(Path::new("priv/js/")));
    mount.mount("/css", Static::new(Path::new("priv/css")));
    mount.mount("/images", Static::new(Path::new("priv/images")));
    mount.mount("/favicon.ico", Static::new(Path::new("priv/favicon.ico")));
    mount.mount("/robots.txt", Static::new(Path::new("priv/robots.txt")));
    
    let mut chain = Chain::new(mount);
    chain.around(login_manager);
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    chain.link_after(ErrorRenderer);
    
    Iron::new(chain).http("0.0.0.0:8080").unwrap();
}