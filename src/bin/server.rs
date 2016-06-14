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
use mount::Mount;
use staticfile::Static;
use router::Router;
use logger::Logger;
use logger::format::Format;
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
use openid_connect::login_manager;
use openid_connect::site_config::*;
use openid_connect::oauth2::*;
use openid_connect::service::routes::login::*;

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


pub fn main() {
    env_logger::init().unwrap();
    let format = Format::new(FORMAT, vec![], vec![]);
    let (logger_before, logger_after) = Logger::new(Some(format.unwrap()));
    
    let user_repo = Arc::new(Box::new(InMemoryUserRepo::new()) as Box<UserRepo>);
    user_repo.add_user(User::new("1".to_owned(), "admin".to_owned(), Some("admin".to_owned()))).unwrap();
    
    let application_repo = Arc::new(Box::new(repos::InMemoryClientApplicationRepo::new()) as Box<repos::ClientApplicationRepo>);
  
    let mut test_app = ClientApplicationBuilder::new();
    test_app.client_name = Some("Wpv WebView Client".to_owned());
    test_app.client_id = Some("wpf.webview.client".to_owned());
    test_app.redirect_uris = Some(vec!["oob://localhost/wpf.webview.client".to_owned()]);
    application_repo.create_client_application(test_app).unwrap();
  
    let mut test_app = ClientApplicationBuilder::new();
    test_app.client_name = Some("pyoidc".to_owned());
    test_app.client_id = Some("pyoidc".to_owned());
    test_app.secret = Some("secret".to_owned());
    test_app.redirect_uris = Some(vec!["oob://localhost/callback".to_owned()]);
    application_repo.create_client_application(test_app).unwrap();
      
    let grant_repo = Arc::new(Box::new(repos::InMemoryGrantRepo::new()) as Box<repos::GrantRepo>);
    
    let token_repo = Arc::new(Box::new(repos::InMemoryTokenRepo::new(user_repo.clone(), grant_repo.clone())) as Box<repos::TokenRepo>);
    
    let cookie_signing_key = b"My secret key"[..].to_owned();
    let mac_signer = MacSigner::new("secret").unwrap();
    
    let sessions = Arc::new(Box::new(sessions::InMemorySessions::new(user_repo.clone())) as Box<sessions::Sessions>);
    let login_manager = login_manager::LoginManager::new(cookie_signing_key);
    let sessions_controller = sessions::SessionController::new(sessions, login_manager.clone());
    
    let config = Config::new(mac_signer, user_repo.clone(), application_repo.clone(), grant_repo.clone(), token_repo.clone(), sessions_controller.clone());
    
    let mut site_config = SiteConfig::new();
    //TODO load site config from file
    site_config.token_issuer = Some("https://localhost:3000".to_owned());
    
    let woidc = openid_config::WellKnownOpenIdConfiguration::new_for_site(&site_config);
    
    let mut router = Router::new();
    router.get("/", web_handler(&config, home_handler));
    router.get("/register", web_handler(&config, register_get_handler));
    router.post("/register", web_handler(&config, register_post_handler));
    router.get("/login", web_handler(&config, login_get_handler));
    router.post("/login", web_handler(&config, login_post_handler));
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
    
    
    
    let mut api_router = Router::new();
    api_router.get("/session", api_handler(&config, session_get_handler));
    api_router.post("/session", api_handler(&config, session_post_handler));
    api_router.delete("/session", api_handler(&config, session_delete_handler));
    
    api_router.get("/applications", api_handler(&config, applications_get_handler));
    api_router.post("/applications", api_handler(&config, applications_post_handler));
    api_router.put("/applications/:id", api_handler(&config, applications_put_handler));
    api_router.delete("/applications/:id", api_handler(&config, applications_delete_handler));
    
    let well_known_router = oauth2::well_known_router(&config);
    
    let oidc_router = oauth2::oauth2_router(&config);
    
    let mut mount = Mount::new();
    mount.mount("/", router);
    mount.mount("/.well-known", well_known_router);
    mount.mount("/api", api_router);
    mount.mount("/connect", oidc_router);
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
    
    outer_chain.link(persistent::Read::<openid_config::WellKnownOpenIdConfiguration>::both(woidc));
    
    outer_chain.link(persistent::Read::<Config>::both(config));
    outer_chain.link(persistent::Read::<SiteConfig>::both(site_config));
    
    Iron::new(outer_chain).http("0.0.0.0:8080").unwrap();
}