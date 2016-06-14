use iron::prelude::*;
use iron::middleware::Handler;
use handlebars_iron::{HandlebarsEngine, DirectorySource};
use router::Router;

use config::Config;
use oauth2::json_error::*;

pub mod routes;
pub mod repos;
pub mod models;
pub mod json_error;

// html content type;
// html error pages
// urlencoded_form accept type?
// form request forgery protection
// TODO move the hbse out to be reused
// TODO macro syntax to wrap several routes similarly
pub fn web_handler<T>(_config: &Config, route: T) -> Chain
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
pub fn api_handler<T>(_config: &Config, route: T) -> Chain
where T: Handler
{
    let mut chain = Chain::new(route);
   
    chain.link_after(JsonErrorRenderer);
    
    chain
}


pub fn well_known_router(config: &Config) -> Router {
    let mut well_known_router = Router::new();
    well_known_router.get("/openid-configuration", api_handler(&config, routes::openid_config_get_handler));
    well_known_router.get("/webfinger", api_handler(&config, routes::webfinger_get_handler));
    well_known_router
}

pub fn oauth2_router(config: &Config) -> Router {
    let mut oidc_router = Router::new();
    oidc_router.get("/authorize", web_handler(&config, routes::authorize_handler));
    oidc_router.get("/complete", web_handler(&config, routes::complete_handler));
    oidc_router.get("/consent", web_handler(&config, routes::consent_get_handler));
    oidc_router.post("/consent", web_handler(&config, routes::consent_post_handler));
    oidc_router.post("/token", api_handler(&config, routes::token_post_handler));
    oidc_router.get("/userinfo", api_handler(&config, routes::userinfo_get_handler));
    oidc_router.get("/identity", api_handler(&config, routes::identity_get_handler));
    oidc_router.post("/register", api_handler(&config, routes::register_application_post_handler));
    oidc_router
}