use std::collections::HashMap;

use iron;
use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;
use urlencoded::UrlEncodedQuery;

use result::{Result, OpenIdConnectError};
use vlad::params::*;
use urls::*;
use ::ResponseType;
use config::Config;


#[derive(Clone, Debug)]
pub struct AuthorizeRequest {
    response_type: ResponseType,
    scopes: Vec<String>, // required. must contain at least "openid" scope.
    client_id: String,
    state: Option<String>, // recommended
    nonce: Option<String>, // optional in authorization code flow. required in implicit flow
    redirect_uri: String, // or url type?
    response_mode: Option<String>, // optional
    prompt: Option<String>,
    display: Option<String>,
    // other stuff: max_age, ui_locales, id_token_hint, login_hint, acr_values
}


impl AuthorizeRequest {
    pub fn from_params(hashmap: &HashMap<String, Vec<String>>) -> Result<AuthorizeRequest> {
        let response_type = try!(multimap_get_one(hashmap, "response_type"));
        let scopes = try!(multimap_get(hashmap, "scope"));
        let client_id = try!(multimap_get_one(hashmap, "client_id"));
        let state = try!(multimap_get_maybe_one(hashmap, "state"));
        let redirect_uri = try!(multimap_get_one(hashmap, "redirect_uri"));
        let prompt = try!(multimap_get_maybe_one(hashmap, "prompt"));
        let display = try!(multimap_get_maybe_one(hashmap, "display"));
        let nonce = try!(multimap_get_maybe_one(hashmap, "nonce"));
        let response_mode = try!(multimap_get_maybe_one(hashmap, "response_mode"));
    
        Ok(AuthorizeRequest {
            response_type: try!(ResponseType::from_str(response_type)),
            scopes: scopes.clone(),
            client_id: client_id.to_owned(),
            state: state.map(|s| s.to_owned()),
            redirect_uri: redirect_uri.to_owned(),
            prompt: prompt.map(|s| s.to_owned()),
            display: display.map(|s| s.to_owned()),
            nonce: nonce.map(|s| s.to_owned()),
            response_mode: response_mode.map(|s| s.to_owned()),
            
        })
    }
    
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().find(|s| *s == scope).is_some()
    }
}

pub fn parse_authorize_request(config: &Config, req: &mut Request) -> Result<AuthorizeRequest> {
    let hashmap = try!(req.get_ref::<UrlEncodedQuery>());
    
    //TODO validate supplied oauth2 params
    
    let auth_req = try!(AuthorizeRequest::from_params(hashmap));
    
    let openid_scope = "openid";
    if !auth_req.has_scope(openid_scope) {
        return Err(OpenIdConnectError::ScopeNotFound(Box::new(openid_scope.to_owned())))
    }
    
    let maybe_client_app = try!(config.application_repo.find_client_application(&auth_req.client_id));
    let client_app = try!(maybe_client_app.ok_or(OpenIdConnectError::ClientApplicationNotFound));
    
    if !client_app.match_redirect_uri(&auth_req.redirect_uri) {
        return Err(OpenIdConnectError::InvalidRedirectUri);
    }
    
    Ok(auth_req)
}

pub fn login_url(req: &mut Request, path: &str, authorize_request: &AuthorizeRequest) -> Result<iron::Url> {
    let mut params = HashMap::new();
    
    if authorize_request.state.is_some() {
      params.insert("state".to_owned(), authorize_request.state.as_ref().unwrap().to_owned());
    }
    
    params.insert("redirect_uri".to_owned(), authorize_request.redirect_uri.clone());
    
    relative_url(req, path, Some(params))
}

/// login with cookie if possible
/// if not logged in or reprompting for credentials redirect to login url
/// otherwise if not got consent or reprompting for consent redirect to consent url
/// otherwise redirect to redirect_uri with code or id_token depending on flow
pub fn authorize_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
    debug!("/authorize");
    let authorize_request = try!(parse_authorize_request(config, req));
    debug!("authorize: {:?}", authorize_request);
    
    // TODO validate subject claim
    // TODO create session and set cookie
    let url = try!(login_url(req, "/login", &authorize_request));
    
    Ok(Response::with((status::Found, Redirect(url))))
}
