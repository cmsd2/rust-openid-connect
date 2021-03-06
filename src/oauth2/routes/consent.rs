use std::collections::HashMap;

use iron;
use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;
use urlencoded::*;
use serde_json::value;
use plugin::Plugin as PluginPlugin;
use jsonwebtoken::json::*;

use rbvt::params::*;
use result::{Result, OpenIdConnectError};
use urls::*;
use config::Config;
use view::View;
use back::*;
use sessions::UserSession;
use oauth2::routes::authorize::{authorize_path, complete_path, auth_redirect_url};
use oauth2::models::*;
use oauth2::repos::*;
use service::routes::login::login_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub name: String,
}

impl Permission {
    pub fn new(name: &str) -> Permission {
        Permission {
            name: name.to_owned(),
        }
    }
}

pub fn permissions_for_scopes(scopes: &[String]) -> Vec<Permission> {
    scopes.iter().map(|s| Permission::new(s)).collect()
}

pub fn consent_redirect_url(req: &mut Request, path: &str, authorize_request: &AuthorizeRequest) -> Result<iron::Url> {
    redirect_forwards_url(req, consent_path(), path, authorize_request.to_params())
}

pub fn consent_path() -> &'static str {
    "/connect/consent"
}

/// called by user agent, redirected from authorize
/// should be logged in with cookie
/// if not logged in redirect to login
/// otherwise render consent page
/// on error set flash error and render consent page form
pub fn consent_get_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    let session = try!(UserSession::eval(req));
    let authenticated = session.map(|s| s.authenticated).unwrap_or(false); 
    let mut view = try!(View::new_for_session("consent.html", req));
     
    let params = try!(req.get::<UrlEncodedQuery>().map_err(OpenIdConnectError::from));
    
    let authorize_request = try!(AuthorizeRequestState::load_from_query(req));
    debug!("consent: {:?}", authorize_request);
                
    if !authenticated {
        let url = { try!(consent_redirect_url(req, login_path(), &authorize_request.request)) };
    
        return Ok(Response::with((status::Found, Redirect(url))));
    }
       
    view.data.insert("permissions".to_owned(), try!(value::to_value(&permissions_for_scopes(&authorize_request.request.scopes)).map_err(OpenIdConnectError::from)));
    view.data.insert("client".to_owned(), try!(value::to_value(&authorize_request.client).map_err(OpenIdConnectError::from)));
    
    let return_token = RedirectToken::new_for_path_and_params(authorize_path(), &authorize_request.request.to_params());
    // view.data.insert("return".to_owned(), value::to_value(&return_token));
    let encoded_token = try!(return_token.encode(&config.mac_signer).map_err(OpenIdConnectError::from));
    view.data.insert("return".to_owned(), try!(value::to_value(&encoded_token).map_err(OpenIdConnectError::from)));

    debug!("parsed query params: {:?}", params);
    
    Ok(Response::with((status::Ok, try!(view.template().map_err(OpenIdConnectError::from)))))
}

/// called by user agent form post
/// should be logged in with cookie
/// if not logged in, redirect to login form with flash error
/// otherwise redirect to caller
/// on error, set flash error and render consent form
pub fn consent_post_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    let session = try!(UserSession::eval(req));
    let authenticated = session.as_ref().map(|s| s.authenticated).unwrap_or(false);
    let params = try!(req.get::<UrlEncodedBody>().map_err(OpenIdConnectError::from));
    let maybe_return_token = try!(load_token(req, &params, "return").map_err(OpenIdConnectError::from));
    let return_token = try!(maybe_return_token.ok_or(OpenIdConnectError::RoutingError("no return token in consent form post".to_owned())).map_err(OpenIdConnectError::from));
    let maybe_authorize_params = try!(return_token.claims.get_value::<HashMap<String,Vec<String>>>("params").map_err(OpenIdConnectError::from));
    let authorize_params = try!(maybe_authorize_params.ok_or(OpenIdConnectError::RoutingError("no authorize payload in consent redirect token".to_owned())).map_err(OpenIdConnectError::from));
     
    let authorize_request = try!(AuthorizeRequestState::load_from_params(req, &authorize_params));
    debug!("consent: {:?}", authorize_request);
    
                
    if !authenticated {
        let url = try!(auth_redirect_url(req, login_path(), &authorize_request.request));
    
        return Ok(Response::with((status::Found, Redirect(url))));
    }
    
    // TODO save granted permissions
    let user_session = try!(session.ok_or(OpenIdConnectError::NoSessionLoaded));
    let user_id = try!(user_session.user_id.ok_or(OpenIdConnectError::UserNotFound));
    let mut update = GrantUpdate::new(user_id, authorize_request.request.client_id.clone());
    update.permissions_added = multimap_get_maybe(&params, "permissions").map(|p| p.clone()).unwrap_or(vec![]);
    try!(config.grant_repo.create_or_update_grant(update));
    
    let redirect_params = return_params(try!(authorize_request.request.encode("authorize", &config.mac_signer)));
    let return_uri = try!(relative_url(req, complete_path(), Some(redirect_params)));

    Ok(Response::with((status::Found, Redirect(return_uri))))
}