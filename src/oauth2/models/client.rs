use std::collections::HashMap;
use std::fmt;

use serde::{Serializer, Deserializer};
use rbvt::params::*;
use jsonwebtoken::validation::*;
use chrono::*;
use jsonwebtoken::claims::time::*;
use cast;

use result::{OpenIdConnectError, Result};
use authentication::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenEndpointAuthMethod {
    ClientSecretPost,
    ClientSecretBasic,
    ClientSecretJwt,
    PrivateKeyJwt,
    None,
}

impl TokenEndpointAuthMethod {
    pub fn from_str(s: &str) -> Result<TokenEndpointAuthMethod> {
        match s {
            "client_secret_post" => Ok(TokenEndpointAuthMethod::ClientSecretPost),
            "client_secret_basic" => Ok(TokenEndpointAuthMethod::ClientSecretBasic),
            "client_secret_jwt" => Ok(TokenEndpointAuthMethod::ClientSecretJwt),
            "private_key_jwt" => Ok(TokenEndpointAuthMethod::PrivateKeyJwt),
            "none" => Ok(TokenEndpointAuthMethod::None),
            _ => Err(OpenIdConnectError::UnknownTokenEndpointAuthMethod(s.to_owned()))
        }
    }
}

impl fmt::Display for TokenEndpointAuthMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            TokenEndpointAuthMethod::ClientSecretBasic => "client_secret_basic",
            TokenEndpointAuthMethod::ClientSecretPost => "client_secret_post",
            TokenEndpointAuthMethod::ClientSecretJwt => "client_secret_jwt",
            TokenEndpointAuthMethod::PrivateKeyJwt => "private_key_jwt",
            TokenEndpointAuthMethod::None => "none",
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientApplication {
    pub client_id: String,
    pub hashed_secret: Option<String>,
    pub client_id_issued_at: Option<u64>,
    pub client_secret_expires_at: Option<u64>, // required if secret issued

    pub redirect_uris: Vec<String>, // required

    // rest are optional
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

impl ClientApplication {
    pub fn new(client_id: String) -> Result<ClientApplication> {
        let now_i64 = try!(SystemTimeProvider.now_utc_minus_a_bit()).timestamp();
        let now = try!(cast::u64(now_i64));
        let client_secret_duration = 3600 * 24 * 365;

        Ok(ClientApplication {
            client_id: client_id,
            hashed_secret: None,
            client_id_issued_at: Some(now),
            client_secret_expires_at: Some(now + client_secret_duration),
            
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
        })
    }
    
    pub fn match_redirect_uri(&self, redirect_uri: &str) -> bool {
        self.redirect_uris.iter().find(|s| &s[..] == redirect_uri).is_some()
    }

    pub fn get_token_endpoint_auth_method(&self) -> Result<TokenEndpointAuthMethod> {
        if let Some(ref auth_method) = self.token_endpoint_auth_method {
            TokenEndpointAuthMethod::from_str(auth_method)
        } else {
            Ok(TokenEndpointAuthMethod::ClientSecretBasic)
        }
    } 

    pub fn uses_secret(&self) -> Result<bool> {
        let auth_method = try!(self.get_token_endpoint_auth_method());
        
        Ok(if auth_method == TokenEndpointAuthMethod::ClientSecretBasic ||
           auth_method == TokenEndpointAuthMethod::ClientSecretPost ||
           auth_method == TokenEndpointAuthMethod::ClientSecretJwt {
            
            true
        } else {
            self.request_object_encryption_alg.is_some()
        })
    }
}

#[derive(Clone, Debug)]
pub struct ClientApplicationBuilder {
    pub client_name: Option<String>,
    pub client_id: Option<String>,
    pub secret: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    
    pub validation_state: ValidationState,
}

impl ClientApplicationBuilder {
    pub fn new() -> ClientApplicationBuilder {
        ClientApplicationBuilder {
            client_name: None,
            client_id: None,
            secret: None,
            redirect_uris: None,
            validation_state: ValidationState::new(),
        }
    }
    
    pub fn build(self) -> Result<ClientApplication> {
        let client_id = try!(self.client_id.ok_or(ValidationError::MissingRequiredValue("client_id".to_owned())));
        let mut app = try!(ClientApplication::new(client_id));
        app.client_name = self.client_name;
        app.redirect_uris = self.redirect_uris.unwrap_or(vec![]);

        Ok(app)
    }
    
    pub fn load_params(&mut self, params: &HashMap<String, Vec<String>>) -> Result<()> {
        self.client_name = try!(multimap_get_maybe_one(params, "client_name")).map(|s| s.to_owned());
        
        if let Some(client_id) = try!(multimap_get_maybe_one(params, "client_id")) {
            self.client_id = Some(client_id.to_owned()); 
        }
        
        self.secret = try!(multimap_get_maybe_one(params, "secret")).map(|s| s.to_owned());
        
        self.redirect_uris = params.get("redirect_uris").map(|r| r.to_owned().into_iter().filter(|r| !r.is_empty()).collect());
        
        Ok(())
    }
    
    pub fn validate(&mut self) -> Result<bool> {
        self.validation_state = ValidationState::new();
        
        if !self.client_id.is_some() {
            self.validation_state.reject("client_id", ValidationError::MissingRequiredValue("client_id".to_owned()));
        }
        
        Ok(self.validation_state.valid)
    }
    
    pub fn build_from_params(params: &HashMap<String, Vec<String>>) -> Result<ClientApplication> {
        let mut builder = ClientApplicationBuilder::new();
        
        try!(builder.load_params(params));
        
        try!(builder.validate());
        
        builder.build()
    }
}
