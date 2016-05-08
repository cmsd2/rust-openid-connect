use iron::prelude::*;
use iron::status;
use iron::typemap;
use iron::mime::Mime;
use persistent;
use serde_json;

use config::Config;
use result::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellKnownOpenIdConfiguration {
    pub issuer: Option<String>,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
    pub userinfo_endpoint: Option<String>,
    pub revocation_endpoint: Option<String>,
    pub jwks_uri: Option<String>,
    pub response_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub scopes_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub claims_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
}

impl WellKnownOpenIdConfiguration {
    pub fn new() -> WellKnownOpenIdConfiguration {
        WellKnownOpenIdConfiguration {
            issuer: None,
            authorization_endpoint: None,
            token_endpoint: None,
            userinfo_endpoint: None,
            revocation_endpoint: None,
            jwks_uri: None,
            response_types_supported: vec![],
            subject_types_supported: vec![],
            id_token_signing_alg_values_supported: vec![],
            scopes_supported: vec![],
            token_endpoint_auth_methods_supported: vec![],
            claims_supported: vec![],
            code_challenge_methods_supported: vec![],
        }
    }
}

impl typemap::Key for WellKnownOpenIdConfiguration {
    type Value = WellKnownOpenIdConfiguration;
}

/// /.well-known/openid-configuration
pub fn openid_config_get_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
    let woidc = try!(req.get::<persistent::Read<WellKnownOpenIdConfiguration>>().map_err(OpenIdConnectError::from));
    
    let body = try!(serde_json::to_string(&woidc).map_err(OpenIdConnectError::from));
    
    let content_type = "application/json".parse::<Mime>().unwrap();
    
    Ok(Response::with((content_type, status::Ok, body)))
}