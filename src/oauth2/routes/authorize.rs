use iron;
use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;
use plugin::Pluggable;
use plugin::Plugin as PluginPlugin;
use url;

use back::*;
use result::{Result, OpenIdConnectError};
use urls::*;
use response_mode::*;
use config::Config;
use sessions::UserSession;
use oauth2::models::authorize_request::*;
use oauth2::routes::consent::consent_path;
use service::routes::login::login_path;

pub fn auth_redirect_url(req: &mut Request, path: &str, authorize_request: &AuthorizeRequest) -> Result<iron::Url> {
    redirect_forwards_url(req, authorize_path(), path, authorize_request.to_params())
}

pub fn auth_consent_url(req: &mut Request, authorize_request: &AuthorizeRequest) -> Result<iron::Url> {
    let path = consent_path();
    
    relative_url(req, path, Some(authorize_request.to_params()))
}

pub fn auth_complete_url(req: &mut Request, authorize_request: &AuthorizeRequest) -> Result<iron::Url> {
    let path = complete_path();
    
    relative_url(req, path, Some(authorize_request.to_params()))
}

pub fn auth_return_to_client_url(req: &mut Request, user_id: &str, authorize_request: &AuthorizeRequest) -> Result<String> {
    let config = try!(Config::get(req));
    let base_uri = &authorize_request.redirect_uri;
    let mut uri = try!(url::Url::parse(base_uri));
    
    let token = try!(config.token_repo.create_code_token(req, user_id, authorize_request));
    
    let query_pairs = try!(token.query_pairs());
    
    if ResponseMode::Query == authorize_request.response_mode.unwrap_or(
            ResponseMode::default_for_response_type(authorize_request.response_type)) {
        uri.set_query(Some(&serialize_query_pairs_vec(query_pairs)));
    } else {
        uri.set_fragment(Some(&serialize_query_pairs_vec(query_pairs)));
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

pub fn authorize_path() -> &'static str {
    "/connect/authorize"
}

pub fn complete_path() -> &'static str {
    "/connect/complete"
}

/// called by user agent on behalf of RP
/// login with cookie if possible
/// if not logged in or reprompting for credentials redirect to login url
/// otherwise if not got consent or reprompting for consent redirect to consent url
/// otherwise redirect to completion url
/// on error either render error or return error response to RP via redirect
pub fn authorize_handler(req: &mut Request) -> IronResult<Response> {
    debug!("/connect/authorize");
    let authorize_request = try!(AuthorizeRequestState::load_from_query(req));
    debug!("authorize: {:?}", authorize_request);
    
    let session = try!(UserSession::eval(req));
    let authenticated = session.map(|s| s.authenticated).unwrap_or(false);
    
    if !authenticated {
        let url = try!(auth_redirect_url(req, login_path(), &authorize_request.request));
    
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
    debug!("/connect/complete");
    let authorize_request = try!(AuthorizeRequestState::load_from_query(req));
    debug!("complete: {:?}", authorize_request);
    
    let session = try!(UserSession::eval(req));
    let authenticated = session.as_ref().map(|s| s.authenticated).unwrap_or(false);
    
    if !authenticated {
        let url = try!(redirect_forwards_url(req, complete_path(), login_path(), authorize_request.request.to_params()));
    
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
    use oauth2::models::*;
    use serde_json;
    use response_type::*;
    
    #[test]
    fn test_auth_serialisation() {
        let auth = AuthorizeRequest::new(ResponseType::new(false, false, false), "client_id#1234567".to_owned(), "redirect_uri#oob".to_owned());

        let s = serde_json::to_string(&auth).unwrap();
        
        assert!(s.find("1234567").is_some());
        assert!(s.find("foo").is_none());
        assert!(s.find("bar").is_none());
    }
    
    #[test]
    fn test_auth_deserialisation() {
        let js = r#"{"response_type":"none", "client_id":"foo", "redirect_uri":"oob", "scopes": ["openid"], "client":{"client_id":"1234567","secret":"bar"}}"#;
        
        let auth = serde_json::from_str::<AuthorizeRequest>(js).unwrap();
        
        assert_eq!(auth.client_id, "foo");
    }
}