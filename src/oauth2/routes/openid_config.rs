use std::sync::Arc;

use iron::prelude::*;
use iron::status;
use iron::typemap;
use iron::mime::Mime;
use persistent;
use serde_json;

use result::*;
use site_config::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellKnownOpenIdConfiguration {
    pub issuer: Option<String>,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
    pub userinfo_endpoint: Option<String>,
    pub revocation_endpoint: Option<String>,
    pub jwks_uri: Option<String>,
    pub registration_endpoint: Option<String>,
    pub response_types_supported: Vec<String>,
    pub response_modes_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
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
            registration_endpoint: None,
            response_types_supported: vec![],
            response_modes_supported: vec![],
            grant_types_supported: vec![],
            subject_types_supported: vec![],
            id_token_signing_alg_values_supported: vec![],
            scopes_supported: vec![],
            token_endpoint_auth_methods_supported: vec![],
            claims_supported: vec![],
            code_challenge_methods_supported: vec![],
        }
    }
    
    pub fn new_for_site(site_config: &SiteConfig) -> WellKnownOpenIdConfiguration {
        let mut c = WellKnownOpenIdConfiguration::new();
        if let Some(ref issuer) = site_config.token_issuer {
            c.issuer = Some(issuer.to_owned());
            c.authorization_endpoint = Some(format!("{}/connect/authorize", issuer));
            c.token_endpoint = Some(format!("{}/connect/token", issuer));
            c.userinfo_endpoint = Some(format!("{}/connect/userinfo", issuer));
            c.jwks_uri = Some(format!("{}/jwks", issuer));
            if site_config.enable_dynamic_client_registration {
                c.registration_endpoint = Some(format!("{}/connect/register", issuer));
            }
            c.response_types_supported = vec![
                "none".to_owned(),
                "code".to_owned(),
                "token".to_owned(),
                "id_token".to_owned(),
                "code token".to_owned(),
                "code id_token".to_owned(),
                "token id_token".to_owned(),
                "code token id_token".to_owned(),
            ];
            c.response_modes_supported = vec![
                "query".to_owned(),
                "fragment".to_owned(),
            ];
            c.grant_types_supported = vec![
                "authorization_code".to_owned(),
                "implicit".to_owned(),
            ];
            c.id_token_signing_alg_values_supported = vec!["HS256".to_owned()]; // must include rs256
            c.scopes_supported = vec!["openid".to_owned()];
            c.subject_types_supported = vec!["pairwise".to_owned(), "public".to_owned()];
        }
        c
    }
    
    pub fn get(req: &mut Request) -> Result<Arc<WellKnownOpenIdConfiguration>> {
        req.get::<persistent::Read<WellKnownOpenIdConfiguration>>().map_err(OpenIdConnectError::from)
    }
}

impl typemap::Key for WellKnownOpenIdConfiguration {
    type Value = WellKnownOpenIdConfiguration;
}

/// /.well-known/openid-configuration
pub fn openid_config_get_handler(req: &mut Request) -> IronResult<Response> {
    let woidc = try!(WellKnownOpenIdConfiguration::get(req));
    
    let body = try!(serde_json::to_string(&woidc).map_err(OpenIdConnectError::from));
    
    let content_type = "application/json".parse::<Mime>().unwrap();
    
    Ok(Response::with((content_type, status::Ok, body)))
}