use std::collections::HashMap;

use serde_json;
use rbvt::params::*;

use result::*;
use oauth2::models::client::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegistrationRequest {
    pub redirect_uris: Vec<String>,
    pub response_types: Vec<String>,
    pub grant_types: Vec<String>,
    pub application_type: Option<String>,
    pub contacts: Vec<String>,
    pub client_name: Option<String>,
    pub client_name_i18n: HashMap<String, String>,
    pub logo_uri: Option<String>,
    pub logo_uri_i18n: HashMap<String, String>,
    pub client_uri: Option<String>,
    pub client_uri_i18n: HashMap<String, String>,
    pub policy_uri: Option<String>,
    pub policy_uri_i18n: HashMap<String, String>,
    pub tos_uri: Option<String>,
    pub tos_uri_i18n: HashMap<String, String>,
    pub jwks_uri: Option<String>,
    pub jwks: Option<String>,
    pub sector_identifier_uri: Option<String>,
    pub subject_type: Option<String>,
    pub id_token_signed_response_alg: Option<String>,
    pub id_token_encrypted_response_alg: Option<String>,
    pub id_token_encrypted_response_enc: Option<String>,
    pub userinfo_signed_response_alg: Option<String>,
    pub userinfo_encrypted_response_alg: Option<String>,
    pub userinfo_encrypted_response_enc: Option<String>,
    pub request_object_signing_alg: Option<String>,
    pub request_object_encryption_alg: Option<String>,
    pub request_object_encryption_enc: Option<String>,
    pub token_endpoint_auth_method: Option<String>,
    pub token_endpoint_auth_signing_alg: Option<String>,
    pub default_max_age: Option<String>,
    pub require_auth_time: Option<bool>,
    pub default_acr_values: Vec<String>,
    pub initiate_login_uri: Option<String>,
    pub request_uris: Vec<String>,
}

impl RegistrationRequest {
    pub fn new() -> RegistrationRequest {
        RegistrationRequest {
            redirect_uris: vec![],
            response_types: vec![],
            grant_types: vec![],
            application_type: None,
            contacts: vec![],
            client_name: None,
            client_name_i18n: HashMap::new(),
            logo_uri: None,
            logo_uri_i18n: HashMap::new(),
            client_uri: None,
            client_uri_i18n: HashMap::new(),
            policy_uri: None,
            policy_uri_i18n: HashMap::new(),
            tos_uri: None,
            tos_uri_i18n: HashMap::new(),
            jwks_uri: None,
            jwks: None,
            sector_identifier_uri: None,
            subject_type: None,
            id_token_signed_response_alg: None,
            id_token_encrypted_response_alg: None,
            id_token_encrypted_response_enc: None,
            userinfo_signed_response_alg: None,
            userinfo_encrypted_response_alg: None,
            userinfo_encrypted_response_enc: None,
            request_object_signing_alg: None,
            request_object_encryption_alg: None,
            request_object_encryption_enc: None,
            token_endpoint_auth_method: None,
            token_endpoint_auth_signing_alg: None,
            default_max_age: None,
            require_auth_time: None,
            default_acr_values: vec![],
            initiate_login_uri: None,
            request_uris: vec![],
        }
    }

    pub fn load_params(&mut self, params: &HashMap<String, Vec<String>>) -> Result<()> {
        self.redirect_uris = params.get("redirect_uris")
            .map(|v| v.to_owned())
            .unwrap_or(vec![])
            .into_iter()
            .filter(|r| !r.is_empty())
            .collect();

        self.client_name = try!(multimap_get_maybe_one(params, "client_name")).map(|s| s.to_owned());

        // ...
        
        Ok(())
    }

    pub fn to_client_builder(self) -> ClientApplicationBuilder {
        unimplemented!()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegistrationResult {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub registration_access_token: Option<String>,
    pub registration_client_uri: Option<String>,
    pub client_id_issued_at: Option<u64>,
    pub client_secret_expires_at: Option<u64>, // required if secret issued

    // plus any other registered metadata
    pub redirect_uris: Vec<String>,
    // ...
}

impl RegistrationResult {
    pub fn from_client(client: ClientApplication, secret: Option<String>) -> RegistrationResult {
        RegistrationResult {
            client_id: client.client_id,
            client_secret: secret,
            client_id_issued_at: client.client_id_issued_at,
            client_secret_expires_at: client.client_secret_expires_at,
            registration_access_token: None,
            registration_client_uri: None,

            redirect_uris: client.redirect_uris,
        }
    }
}