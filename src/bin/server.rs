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
extern crate persistent;
extern crate jsonwebtoken;

#[macro_use] extern crate log;
extern crate env_logger;

use std::sync::Arc;
use std::path::Path;

use iron::prelude::*;
use iron::{AfterMiddleware};
use iron::middleware::Handler;
use iron::mime::Mime;
use mount::Mount;
use staticfile::Static;
use router::Router;
use logger::Logger;
use logger::format::Format;
use handlebars_iron::{HandlebarsEngine, DirectorySource};
use jsonwebtoken::crypto::mac_signer::MacSigner;

use openid_connect::routes::home::*;
use openid_connect::routes::register::*;
use openid_connect::routes::application_api::*;
use openid_connect::routes::session::*;
use openid_connect::routes::applications;
use openid_connect::routes::grants;
use openid_connect::users::*;
use openid_connect::config::*;
use openid_connect::oauth2;
use openid_connect::oauth2::routes::openid_config;
use openid_connect::oauth2::models::client::*;
use openid_connect::sessions;
use openid_connect::service;
use openid_connect::login_manager;
use openid_connect::result::OpenIdConnectError;
use openid_connect::site_config::*;

// without colours so it works on conhost terminals
static FORMAT: &'static str =
        "{method} {uri} -> {status} ({response-time})";
   
struct ErrorRenderer;

impl AfterMiddleware for ErrorRenderer {
    /// render error as human readable error page
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
        debug!("{:?} caught in ErrorRecover AfterMiddleware.", &err);
        
        let new_body = format!("{}", err);
        
        Ok(err.response.set(new_body))
    }
}

struct JsonErrorRenderer;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ErrorView {
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

pub fn main() {
    env_logger::init().unwrap();
    let format = Format::new(FORMAT, vec![], vec![]);
    let (logger_before, logger_after) = Logger::new(Some(format.unwrap()));
    
    let user_repo = Arc::new(Box::new(InMemoryUserRepo::new()) as Box<UserRepo>);
    user_repo.add_user(User::new("1".to_owned(), "admin".to_owned(), Some("admin".to_owned()))).unwrap();
    
    let application_repo = Arc::new(Box::new(oauth2::InMemoryClientApplicationRepo::new()) as Box<oauth2::ClientApplicationRepo>);
    let mut test_app = ClientApplicationBuilder::new();
    test_app.name = Some("Wpv WebView Client".to_owned());
    test_app.client_id = Some("wpf.webview.client".to_owned());
    test_app.redirect_uris = Some(vec!["oob://localhost/wpf.webview.client".to_owned()]);
    application_repo.create_client_application(test_app).unwrap();
    
    let grant_repo = Arc::new(Box::new(oauth2::InMemoryGrantRepo::new()) as Box<oauth2::GrantRepo>);
    
    let token_repo = Arc::new(Box::new(oauth2::InMemoryTokenRepo::new(user_repo.clone(), grant_repo.clone())) as Box<oauth2::TokenRepo>);
    
    let cookie_signing_key = b"My secret key"[..].to_owned();
    let mac_signer = MacSigner::new("secret").unwrap();
    
    let sessions = Arc::new(Box::new(sessions::InMemorySessions::new(user_repo.clone())) as Box<sessions::Sessions>);
    let login_manager = login_manager::LoginManager::new(cookie_signing_key);
    let sessions_controller = sessions::SessionController::new(sessions, login_manager.clone());
    
    let config = Config::new(mac_signer, user_repo.clone(), application_repo.clone(), grant_repo.clone(), token_repo.clone(), sessions_controller.clone());
    
    let site_config = SiteConfig::new();
    
    // html content type;
    // html error pages
    // urlencoded_form accept type?
    // form request forgery protection
    // TODO move the hbse out to be reused
    // TODO macro syntax to wrap several routes similarly
    fn web_handler<T>(_config: &Config, route: T) -> Chain
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
    fn api_handler<T>(_config: &Config, route: T) -> Chain
    where T: Handler
    {
        let mut chain = Chain::new(route);
        
        chain.link_after(JsonErrorRenderer);
        
        chain
    }
    
    
    let mut router = Router::new();
//    router.get("/.well-known/)
    router.get("/authorize", web_handler(&config, oauth2::authorize_handler));
    router.get("/complete", web_handler(&config, oauth2::complete_handler));
    router.get("/", web_handler(&config, home_handler));
    router.get("/login", web_handler(&config, service::login_get_handler));
    router.post("/login", web_handler(&config, service::login_post_handler));
    router.get("/consent", web_handler(&config, service::consent_get_handler));
    router.post("/consent", web_handler(&config, service::consent_post_handler));
    router.get("/register", web_handler(&config, register_get_handler));
    router.post("/register", web_handler(&config, register_post_handler));
    
    router.get("/applications", web_handler(&config, applications::applications_index_handler));
    router.get("/applications/new", web_handler(&config, applications::applications_new_handler));
    router.get("/applications/:id", web_handler(&config, applications::applications_show_handler));
    router.get("/applications/:id/edit", web_handler(&config, applications::applications_edit_handler));
    router.post("/applications/:id", web_handler(&config, applications::applications_update_handler));
    router.post("/applications", web_handler(&config, applications::applications_create_handler));
    /*router.get("/applications/:id/delete", web_handler(&config, applications::applications_delete_handler));
    router.post("/applications/:id/delete", web_handler(&config, applications::applications_delete_handler));*/
    
    router.get("/grants", web_handler(&config, grants::grants_index_handler));
    router.get("/grants/:id", web_handler(&config, grants::grants_show_handler));
    router.get("/grants/:id/edit", web_handler(&config, grants::grants_edit_handler));
    router.post("/grants/:id", web_handler(&config, grants::grants_update_handler));
    //TODO delete
    
    router.post("/token", api_handler(&config, oauth2::token_post_handler));
    router.get("/userinfo", api_handler(&config, oauth2::userinfo_get_handler));
    router.get("/identity", api_handler(&config, oauth2::identity_get_handler));
    
    let mut api_router = Router::new();
    api_router.get("/session", api_handler(&config, session_get_handler));
    api_router.post("/session", api_handler(&config, session_post_handler));
    api_router.delete("/session", api_handler(&config, session_delete_handler));
    
    api_router.get("/applications", api_handler(&config, applications_get_handler));
    api_router.post("/applications", api_handler(&config, applications_post_handler));
    api_router.put("/applications/:id", api_handler(&config, applications_put_handler));
    api_router.delete("/applications/:id", api_handler(&config, applications_delete_handler));
    
    let mut well_known_router = Router::new();
    well_known_router.get("/openid-configuration", api_handler(&config, oauth2::openid_config_get_handler));
    
    let mut mount = Mount::new();
    mount.mount("/", router);
    mount.mount("/.well-known", well_known_router);
    mount.mount("/api", api_router);
    mount.mount("/js", Static::new(Path::new("web/priv/js/")));
    mount.mount("/css", Static::new(Path::new("web/priv/css")));
    mount.mount("/images", Static::new(Path::new("web/priv/images")));
    mount.mount("/favicon.ico", Static::new(Path::new("web/priv/favicon.ico")));
    mount.mount("/robots.txt", Static::new(Path::new("web/priv/robots.txt")));
    
    let mut chain = Chain::new(mount);
    chain.link_before(sessions_controller);
    
    let mut outer_chain = Chain::new(chain);
    outer_chain.around(login_manager);
    outer_chain.link_before(logger_before);
    outer_chain.link_after(logger_after);
    outer_chain.link_after(ErrorRenderer);
    
    let woidc = openid_config::WellKnownOpenIdConfiguration::new();
    outer_chain.link(persistent::Read::<openid_config::WellKnownOpenIdConfiguration>::both(woidc));
    
    outer_chain.link(persistent::Read::<Config>::both(config));
    outer_chain.link(persistent::Read::<SiteConfig>::both(site_config));
    
    Iron::new(outer_chain).http("0.0.0.0:8080").unwrap();
}