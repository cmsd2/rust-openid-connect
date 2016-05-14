use std::collections::HashMap;

use iron;
use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;
use plugin::Pluggable;
use plugin::Plugin as PluginPlugin;
use urlencoded::UrlEncodedQuery;
use url;

use jsonwebtoken;
use jsonwebtoken::jwt::*;
use jsonwebtoken::signer::*;
use jsonwebtoken::verifier::*;
use jsonwebtoken::header::*;
use jsonwebtoken::algorithm::*;
use back::*;
use result::{Result, OpenIdConnectError};
use rbvt::params::*;
use rbvt::state::*;
use rbvt::result::ValidationError;
use urls::*;
use response_type::ResponseType;
use response_mode::*;
use config::Config;
use oauth2::{ClientApplication, ClientApplicationRepo};
use sessions::UserSession;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AuthorizeStep {
    Authorize,
    Login,
    Consent,
    Complete,
}

#[derive(Clone, Debug)]
pub struct AuthorizeRequestState {
    pub request: AuthorizeRequest,
    pub step: AuthorizeStep,
    pub client: Option<ClientApplication>,
}

impl AuthorizeRequestState {
    pub fn new(request: AuthorizeRequest) -> AuthorizeRequestState {
        AuthorizeRequestState {
            request: request,
            step: AuthorizeStep::Authorize,
            client: None,
        }
    }
    
    pub fn validate(&self, validation_state: &mut ValidationState) -> Result<bool> { 
        let openid_scope = "openid";
        if !self.request.has_scope(openid_scope) {
            validation_state.reject("scope", ValidationError::MissingRequiredValue("scope: openid".to_owned()));
        }
        
        if let Some(ref client) = self.client {
            if !client.match_redirect_uri(&self.request.redirect_uri) {
                validation_state.reject("redirect_uri", ValidationError::InvalidValue("redirect_uri does not match".to_owned()));
            }
        } else {
            validation_state.reject("client_id", ValidationError::InvalidValue("client not found for client_id".to_owned()));
        }
        
        if let Some(response_mode) = self.request.response_mode {
            if let Err(e) = ResponseMode::validate_response_mode(response_mode, self.request.response_type) {
                validation_state.reject("response_mode", ValidationError::InvalidValue(e.to_string()));
            }
        }
        
        Ok(validation_state.valid)
    }
    
    pub fn load_client(&mut self, client_repo: &ClientApplicationRepo) -> Result<()> {
        self.client = try!(client_repo.find_client_application(&self.request.client_id));

        Ok(())
    }
    
    pub fn load_from_query(req: &mut Request) -> Result<AuthorizeRequestState> {
        let hashmap = try!(req.get::<UrlEncodedQuery>());
        
        Self::load_from_params(req, &hashmap)
    }
    
    pub fn load_from_params(req: &mut Request, hashmap: &HashMap<String, Vec<String>>) -> Result<AuthorizeRequestState> {
        let config = try!(Config::get(req));
        
        let auth_req = if let Some(jwt_req) = try!(multimap_get_maybe_one(hashmap, "request")) {
            try!(AuthorizeRequest::decode(&jwt_req, &config.mac_signer))
        } else {
            try!(AuthorizeRequest::from_params(hashmap))
        };
        
        let mut auth_req_state = AuthorizeRequestState::new(auth_req);
    
        try!(auth_req_state.load_client(&**config.application_repo));        
        
        let mut validation_state = ValidationState::new();
        
        if ! try!(auth_req_state.validate(&mut validation_state)) {
            return Err(OpenIdConnectError::ValidationError(ValidationError::ValidationError(validation_state)));
        }
    
        Ok(auth_req_state)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthorizeRequest {
    pub iss: Option<String>,
    pub aud: Option<String>,
    pub response_type: ResponseType,
    pub scopes: Vec<String>, // required. must contain at least "openid" scope.
    pub client_id: String,
    state: Option<String>, // recommended
    nonce: Option<String>, // optional in authorization code flow. required in implicit flow
    pub redirect_uri: String, // or url type?
    pub response_mode: Option<ResponseMode>, // optional
    pub prompt: Option<String>,
    pub display: Option<String>,
    // other stuff: max_age, ui_locales, id_token_hint, login_hint, acr_values
}

impl AuthorizeRequest {
    pub fn new(response_type: ResponseType, client_id: String, redirect_uri: String) -> AuthorizeRequest {
        AuthorizeRequest {
            iss: None,
            aud: None,
            response_type: response_type,
            scopes: vec![],
            client_id: client_id,
            state: None,
            nonce: None,
            redirect_uri: redirect_uri,
            response_mode: None,
            prompt: None,
            display: None,
        }
    }
    
    pub fn to_params(&self) -> HashMap<String, Vec<String>> {
        let mut params = HashMap::new();
        if self.iss.is_some() {
            params.insert("iss".to_owned(), vec![self.iss.as_ref().unwrap().to_owned()]);
        }
        if self.aud.is_some() {
            params.insert("aud".to_owned(), vec![self.aud.as_ref().unwrap().to_owned()]);
        }
        params.insert("response_type".to_owned(), vec![self.response_type.to_string()]);
        params.insert("scope".to_owned(), self.scopes.clone());
        params.insert("client_id".to_owned(), vec![self.client_id.clone()]);
        if self.state.is_some() {
            params.insert("state".to_owned(), vec![self.state.as_ref().unwrap().to_owned()]);
        }
        if self.nonce.is_some() {
            params.insert("nonce".to_owned(), vec![self.nonce.as_ref().unwrap().to_owned()]);
        }
        params.insert("redirect_uri".to_owned(), vec![self.redirect_uri.clone()]);
        if self.response_mode.is_some() {
            params.insert("response_mode".to_owned(), vec![self.response_mode.as_ref().unwrap().to_string()]);
        }
        if self.prompt.is_some() {
            params.insert("prompt".to_owned(), vec![self.prompt.as_ref().unwrap().to_owned()]);
        }
        if self.display.is_some() {
            params.insert("display".to_owned(), vec![self.display.as_ref().unwrap().to_owned()]);
        }

        params
    }
    
    pub fn from_params(hashmap: &HashMap<String, Vec<String>>) -> Result<AuthorizeRequest> {
        let iss = try!(multimap_get_maybe_one(hashmap, "iss"));
        let aud = try!(multimap_get_maybe_one(hashmap, "aud"));
        let response_type = try!(multimap_get_one(hashmap, "response_type"));
        let scopes = try!(multimap_get(hashmap, "scope"));
        let client_id = try!(multimap_get_one(hashmap, "client_id"));
        let state = try!(multimap_get_maybe_one(hashmap, "state"));
        let redirect_uri = try!(multimap_get_one(hashmap, "redirect_uri"));
        let prompt = try!(multimap_get_maybe_one(hashmap, "prompt"));
        let display = try!(multimap_get_maybe_one(hashmap, "display"));
        let nonce = try!(multimap_get_maybe_one(hashmap, "nonce"));
        let maybe_response_mode_str = try!(multimap_get_maybe_one(hashmap, "response_mode"));
        let response_mode = if let Some(response_mode_str) = maybe_response_mode_str {
            Some(try!(ResponseMode::from_str(response_mode_str)))
        } else {
            None
        };
    
        Ok(AuthorizeRequest {
            iss: iss.map(|s| s.to_owned()),
            aud: aud.map(|s| s.to_owned()),
            response_type: try!(ResponseType::from_str(response_type)),
            scopes: scopes.clone(),
            client_id: client_id.to_owned(),
            state: state.map(|s| s.to_owned()),
            redirect_uri: redirect_uri.to_owned(),
            prompt: prompt.map(|s| s.to_owned()),
            display: display.map(|s| s.to_owned()),
            nonce: nonce.map(|s| s.to_owned()),
            response_mode: response_mode.clone(),
        })
    }
    
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().find(|s| *s == scope).is_some()
    }
    
    pub fn encode<S: Signer>(&self, jwt_type: &str, signer: &S) -> Result<String> {
        let mut header = Header::new(Algorithm::HS256);
        header.typ = Some(jwt_type.to_owned());
        jsonwebtoken::encode(header, self, signer).map_err(OpenIdConnectError::from)
    }
    
    pub fn decode<V: Verifier>(encoded: &str, verifier: &V) -> Result<AuthorizeRequest> {
        let token_data = try!(jsonwebtoken::decode(encoded, verifier));
        
        Ok(token_data.claims)
    }
}

pub fn auth_redirect_url(req: &mut Request, path: &str, authorize_request: &AuthorizeRequest) -> Result<iron::Url> {
    redirect_forwards_url(req, "/authorize", path, authorize_request.to_params())
}

pub fn auth_consent_url(req: &mut Request, authorize_request: &AuthorizeRequest) -> Result<iron::Url> {
    let path = "/consent";
    
    relative_url(req, path, Some(authorize_request.to_params()))
}

pub fn auth_complete_url(req: &mut Request, authorize_request: &AuthorizeRequest) -> Result<iron::Url> {
    let path = "/complete";
    
    relative_url(req, path, Some(authorize_request.to_params()))
}

pub fn auth_return_to_client_url(req: &mut Request, user_id: &str, authorize_request: &AuthorizeRequest) -> Result<String> {
    let config = try!(Config::get(req));
    let base_uri = &authorize_request.redirect_uri;
    let mut uri = try!(url::Url::parse(base_uri));
    
    //let mut query_pairs = uri.query_pairs_mut();
    let mut query_pairs = vec![];
    // query_pairs.clear();
    
    //TODO generate real tokens
    
    if authorize_request.response_type.code {
        // query_pairs.append_pair("code", "blah");
        query_pairs.push(("code".to_owned(), "blah".to_owned()));
    }
    
    if authorize_request.response_type.id_token {
        let claims = try!(config.token_repo.get_user_claims(req, user_id, &authorize_request.client_id, &authorize_request.scopes));
        let id_token = Jwt::new(Header::default(), claims); //TODO use RSA
        query_pairs.push(("id_token".to_owned(), try!(id_token.encode(&config.mac_signer))));
    }
    
    if authorize_request.response_type.token {
        // query_pairs.append_pair("token", "blah");
        query_pairs.push(("token".to_owned(), "blah".to_owned()));
    }
    
    if ResponseMode::Query == authorize_request.response_mode.unwrap_or(
            ResponseMode::default_for_response_type(authorize_request.response_type)) {
        uri.set_query_from_pairs(query_pairs);
    } else {
        uri.fragment = Some(url::form_urlencoded::serialize(query_pairs));
    }
      
    Ok(uri.to_string())
}

pub fn should_prompt(authorize_request: &AuthorizeRequest) -> bool {
    // TODO match boolean truthy strings?
    if authorize_request.prompt.as_ref().map(|s| &s[..]).unwrap_or("") == "true" {
        return true;
    }
    
    // TODO match requested scopes against granted scopes
    
    true
}

/// called by user agent on behalf of RP
/// login with cookie if possible
/// if not logged in or reprompting for credentials redirect to login url
/// otherwise if not got consent or reprompting for consent redirect to consent url
/// otherwise redirect to completion url
/// on error either render error or return error response to RP via redirect
pub fn authorize_handler(req: &mut Request) -> IronResult<Response> {
    debug!("/authorize");
    let authorize_request = try!(AuthorizeRequestState::load_from_query(req));
    debug!("authorize: {:?}", authorize_request);
    
    let session = try!(UserSession::eval(req));
    let authenticated = session.map(|s| s.authenticated).unwrap_or(false);
    
    if !authenticated {
        let url = try!(auth_redirect_url(req, "/login", &authorize_request.request));
    
        Ok(Response::with((status::Found, Redirect(url))))
    } else if should_prompt(&authorize_request.request) {
        let consent_url = try!(auth_consent_url(req, &authorize_request.request));
        
        Ok(Response::with((status::Found, Redirect(consent_url))))
    } else {
        let complete_url = try!(auth_complete_url(req, &authorize_request.request));
        
        Ok(Response::with((status::Found, Redirect(complete_url))))
    }
}

/// called by user agent after logging in and giving consent
/// login with cookie if possible
/// if not logged in or reprompting for credentials redirect to login url
/// otherwise redirect to redirect_uri with code or id_token depending on flow
/// on error either render error or return error response to RP via redirect
pub fn complete_handler(req: &mut Request) -> IronResult<Response> {
    debug!("/complete");
    let authorize_request = try!(AuthorizeRequestState::load_from_query(req));
    debug!("complete: {:?}", authorize_request);
    
    let session = try!(UserSession::eval(req));
    let authenticated = session.as_ref().map(|s| s.authenticated).unwrap_or(false);
    
    if !authenticated {
        let url = try!(redirect_forwards_url(req, "/complete", "/login", authorize_request.request.to_params()));
    
        Ok(Response::with((status::Found, Redirect(url))))
    } else {
        let user_session = try!(session.ok_or(OpenIdConnectError::NoSessionLoaded));
        let user_id = try!(user_session.user_id.ok_or(OpenIdConnectError::UserNotFound));
    
        Ok(Response::with((status::Found, RoidcRedirectRaw(try!(auth_return_to_client_url(req, &user_id, &authorize_request.request))))))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use oauth2::models::client::*;
    use serde_json;
    use response_type::*;
    
    #[test]
    fn test_client_app_is_not_serialised() {
        let mut auth = AuthorizeRequest::new(ResponseType::new(false, false, false), "client_id#1234567".to_owned(), "redirect_uri#oob".to_owned());
        auth.client = Some(ClientApplication::new("id#foo".to_owned(), Some("secret#bar".to_owned())));
        
        let s = serde_json::to_string(&auth).unwrap();
        
        assert!(s.find("1234567").is_some());
        assert!(s.find("foo").is_none());
        assert!(s.find("bar").is_none());
    }
    
    #[test]
    fn test_client_app_is_not_deserialised() {
        let js = r#"{"response_type":"none", "client_id":"foo", "redirect_uri":"oob", "scopes": ["openid"], "client":{"client_id":"1234567","secret":"bar"}}"#;
        
        let auth = serde_json::from_str::<AuthorizeRequest>(js).unwrap();
        
        assert_eq!(auth.client_id, "foo");
        assert!(auth.client.is_none());
    }
}