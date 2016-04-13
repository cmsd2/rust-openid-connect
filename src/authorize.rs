use std::collections::HashMap;

use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;
use urlencoded::UrlEncodedQuery;
use handlebars_iron::Template;

use result::{Result, OpenIdConnectError};
use params::*;
use urls::*;
use ::ResponseType;


#[derive(Clone, Debug)]
pub struct AuthorizeRequest {
    response_type: ResponseType,
    scopes: Vec<String>,
    client_id: String,
    state: String,
    // nonce: String, // ?
    redirect_uri: String, // or url type?
    prompt: Option<String>,
}

impl AuthorizeRequest {
    pub fn from_params(hashmap: &HashMap<String, Vec<String>>) -> Result<AuthorizeRequest> {
        let response_type = try!(multimap_get_one(hashmap, "response_type"));
        let scopes = try!(multimap_get(hashmap, "scope"));
        let client_id = try!(multimap_get_one(hashmap, "client_id"));
        let state = try!(multimap_get_one(hashmap, "state"));
        let redirect_uri = try!(multimap_get_one(hashmap, "redirect_uri"));
        let prompt = try!(multimap_get_maybe_one(hashmap, "prompt"));
    
        Ok(AuthorizeRequest {
            response_type: try!(ResponseType::from_str(response_type)),
            scopes: scopes.clone(),
            client_id: client_id.to_owned(),
            state: state.to_owned(),
            redirect_uri: redirect_uri.to_owned(),
            prompt: prompt.map(|s| s.to_owned()),
        })
    }
    
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().find(|s| *s == scope).is_some()
    }
}

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
    
    // TODO validate subject claim
    // TODO create session and set cookie
    let url = try!(relative_url(req, "/login"));
    
    Ok(Response::with((status::Found, Redirect(url))))
}
