use std::collections::HashMap;

use iron;
use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;
use plugin::{Extensible, Pluggable};
use plugin::Plugin as PluginPlugin;
use urlencoded::UrlEncodedQuery;

use jsonwebtoken;
use jsonwebtoken::signer::*;
use jsonwebtoken::verifier::*;
use jsonwebtoken::header::*;
use jsonwebtoken::algorithm::*;
use result::{Result, OpenIdConnectError};
use rbvt::params::*;
use rbvt::state::*;
use rbvt::result::ValidationError;
use urls::*;
use response_type::ResponseType;
use config::Config;
use oauth2::{ClientApplication, ClientApplicationRepo};
use sessions::UserSession;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthorizeRequest {
    pub response_type: ResponseType,
    pub scopes: Vec<String>, // required. must contain at least "openid" scope.
    pub client_id: String,
    state: Option<String>, // recommended
    nonce: Option<String>, // optional in authorization code flow. required in implicit flow
    pub redirect_uri: String, // or url type?
    pub response_mode: Option<String>, // optional
    pub prompt: Option<String>,
    pub display: Option<String>,
    // other stuff: max_age, ui_locales, id_token_hint, login_hint, acr_values
    
    #[serde(skip_serializing, skip_deserializing)]
    pub client: Option<ClientApplication>,
    
    // #[serde(skip_serializing, skip_deserializing)]
    // #[serde(skip_serializing)]
    // validation_state: Option<ValidationState>,
}


impl AuthorizeRequest {
    pub fn new(response_type: ResponseType, client_id: String, redirect_uri: String) -> AuthorizeRequest {
        AuthorizeRequest {
            response_type: response_type,
            scopes: vec![],
            client_id: client_id,
            state: None,
            nonce: None,
            redirect_uri: redirect_uri,
            response_mode: None,
            prompt: None,
            display: None,
            
            client: None,
            // validation_state: ValidationState::default()
        }
    }
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
            
            client: None,
            
            // validation_state: ValidationState::new(),
        })
    }
    
    pub fn load_client(&mut self, client_repo: &ClientApplicationRepo) -> Result<()> {
        self.client = try!(client_repo.find_client_application(&self.client_id));

        Ok(())
    }
    
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().find(|s| *s == scope).is_some()
    }
    
    pub fn validate(&mut self, validation_state: &mut ValidationState) -> Result<bool> {
        //self.validation_state = ValidationState::new();
        
        let openid_scope = "openid";
        if !self.has_scope(openid_scope) {
            validation_state.reject("scope", ValidationError::MissingRequiredValue("scope: openid".to_owned()));
        }
        
        if let Some(ref client) = self.client {
            if !client.match_redirect_uri(&self.redirect_uri) {
                validation_state.reject("redirect_uri", ValidationError::InvalidValue("redirect_uri does not match".to_owned()));
            }
        } else {
            validation_state.reject("client_id", ValidationError::InvalidValue("client not found for client_id".to_owned()));
        }
        
        Ok(validation_state.valid)
    }
    
    pub fn load_from_query(req: &mut Request) -> Result<AuthorizeRequest> {
        let config = try!(Config::get(req));
        
        let hashmap = try!(req.get_ref::<UrlEncodedQuery>());
        
        let mut auth_req = if let Some(jwt_req) = try!(multimap_get_maybe_one(hashmap, "jwt_req")) {
            try!(AuthorizeRequest::decode(&jwt_req, &config.mac_signer))
        } else {
            try!(AuthorizeRequest::from_params(hashmap))
        };
    
        try!(auth_req.load_client(&**config.application_repo));
    
        let mut validation_state = ValidationState::new();
        
        if ! try!(auth_req.validate(&mut validation_state)) {
            return Err(OpenIdConnectError::ValidationError(ValidationError::ValidationError(validation_state)));
        }
    
        Ok(auth_req)
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
    let config = try!(Config::get(req));
    
    let mut params = HashMap::new();
    
    params.insert("return".to_owned(), try!(authorize_request.encode("authorize", &config.mac_signer)));

    relative_url(req, path, Some(params))
}

pub fn auth_complete_url(req: &mut Request, authorize_request: &AuthorizeRequest) -> Result<iron::Url> {
    unimplemented!()
}

pub fn should_prompt(authorize_request: &AuthorizeRequest) -> bool {
    true
}

/// called by user agent on behalf of RP
/// login with cookie if possible
/// if not logged in or reprompting for credentials redirect to login url
/// otherwise if not got consent or reprompting for consent redirect to consent url
/// otherwise redirect to redirect_uri with code or id_token depending on flow
/// on error either render error or return error response to RP via redirect
pub fn authorize_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    
    debug!("/authorize");
    let authorize_request = try!(AuthorizeRequest::load_from_query(req));
    debug!("authorize: {:?}", authorize_request);
    
    let session = try!(UserSession::eval(req));
    let authenticated = session.map(|s| s.authenticated).unwrap_or(false);
    
    if !authenticated {
        let url = try!(auth_redirect_url(req, "/login", &authorize_request));
    
        Ok(Response::with((status::Found, Redirect(url))))
    } else if should_prompt(&authorize_request) {
        let consent_url = try!(auth_redirect_url(req, "/consent", &authorize_request));
        
        Ok(Response::with((status::Found, Redirect(consent_url))))
    } else {
        let complete_url = try!(auth_complete_url(req, &authorize_request));
        
        Ok(Response::with((status::Found, Redirect(complete_url))))
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