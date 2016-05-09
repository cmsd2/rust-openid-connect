use std::collections::HashMap;

use iron::prelude::*;
use iron::status;
use urlencoded::UrlEncodedBody;
use serde_json;

use result::{Result, OpenIdConnectError};
use rbvt::params::*;
use rbvt::result::*;
use rbvt::state::*;
use rbvt::builder::*;
use oauth2::models::tokens::{Token, TokenType};
use authentication;
use jsonwebtoken::*;
use jsonwebtoken::jwt::*;
use jsonwebtoken::signer::*;
use jsonwebtoken::crypto::mac_signer::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GrantType {
    AuthorizationCode,
    ClientCredentials,
}

impl GrantType {
    pub fn from_str(s: &str) -> Result<GrantType> {
        match s {
            "authorization_code" => Ok(GrantType::AuthorizationCode),
            "client_credentials" => Ok(GrantType::ClientCredentials),
            _ => Err(OpenIdConnectError::UnknownGrantType(Box::new(s.to_owned())))
        }
    }
}

#[derive(Clone, Debug)]
pub struct TokenRequest {
    grant_type: GrantType,
    code: Option<String>,
    redirect_uri: Option<String>,
}

impl TokenRequest {
    pub fn new(grant_type: GrantType, code: Option<String>, redirect_uri: Option<String>) -> TokenRequest {
        TokenRequest {
            grant_type: grant_type,
            code: code,
            redirect_uri: redirect_uri,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TokenRequestBuilder {
    grant_type: Option<String>,
    code: Option<String>,
    redirect_uri: Option<String>,
    
    validation_state: ValidationState,
}

impl TokenRequestBuilder {
    pub fn new() -> TokenRequestBuilder {
        TokenRequestBuilder {
            grant_type: None,
            code: None,
            redirect_uri: None,
            
            validation_state: ValidationState::new(),
        }
    }
    
    pub fn build(self) -> Result<TokenRequest> {
        if self.validation_state.valid {
            Ok(TokenRequest {
                grant_type: try!(
                    self.grant_type
                        .ok_or(OpenIdConnectError::from(ValidationError::MissingRequiredValue("grant_type".to_owned())))
                        .and_then(|gt| GrantType::from_str(&gt))),
                code: self.code,
                redirect_uri: self.redirect_uri,
            })
        } else {
            Err(OpenIdConnectError::from(ValidationError::ValidationError(self.validation_state)))
        }
    }
    
    pub fn load_params(&mut self, params: &HashMap<String, Vec<String>>) -> Result<()> {
        self.grant_type = try!(multimap_get_maybe_one(params, "grant_type")).map(|s| s.to_owned()); 
        self.code = try!(multimap_get_maybe_one(params, "code")).map(|s| s.to_owned());
            
        Ok(())
    }
    
    pub fn validate(&mut self) -> Result<bool> {
        self.validation_state = ValidationState::new();
        
        if let Some(ref grant_type_str) = self.grant_type {
            if let Ok(grant_type) = GrantType::from_str(grant_type_str) {
                if self.code.is_none() && grant_type == GrantType::AuthorizationCode {
                    self.validation_state.reject("code", ValidationError::MissingRequiredValue("code".to_owned()));
                }
                
                if self.redirect_uri.is_none() && grant_type != GrantType::ClientCredentials {
                    self.validation_state.reject("redirect_uri", ValidationError::MissingRequiredValue("redirect_uri".to_owned()));
                }
            } else {
                self.validation_state.reject("grant_type", ValidationError::InvalidValue("grant_type".to_owned()));
            }
        } else {
            self.validation_state.reject("grant_type", ValidationError::MissingRequiredValue("grant_type".to_owned()));
        }
        
        Ok(self.validation_state.valid)
    }
    
    pub fn build_from_params(hashmap: &HashMap<String, Vec<String>>) -> Result<TokenRequest> {
        let mut builder = TokenRequestBuilder::new();

        try!(builder.load_params(hashmap));
        
        try!(builder.validate());
        
        builder.build()
    }
    
    pub fn build_from_request(req: &mut Request) -> Result<TokenRequest> {
        let hashmap = try!(req.get_ref::<UrlEncodedBody>());
        debug!("token request body: {:?}", hashmap);
    
        let token_request = try!(Self::build_from_params(hashmap));
    
        Ok(token_request)
    }
}

#[derive(Clone, Debug)]
pub struct TokenResponse {
    access_token: String,
    token_type: TokenType,
    refresh_token: String,
    expires_in: u32,
    id_token: String,
}

#[derive(Clone, Debug)]
pub struct TokenErrorResponse;

/// called by RP server
/// exchange code for access_token, id_token and maybe refresh_token
/// on error render error response
pub fn token_post_handler(req: &mut Request) -> IronResult<Response> {
    debug!("/token");
    
    let token_request = try!(TokenRequestBuilder::build_from_request(req));
    debug!("token request: {:?}", token_request);
    
    //TODO move this all somewhere else 
    let mut jwt = Jwt::new();
    jwt.claims.set_value("iss", &"roidc");
    jwt.claims.set_value("nonce", &authentication::new_nonce());
    
    let signer = try!(MacSigner::new("secret").map_err(OpenIdConnectError::from));
    
    let id_token = try!(jwt.encode(&signer).map_err(OpenIdConnectError::from));
    
    let token = Token {
        access_token: authentication::new_secret(),
        token_type: TokenType::Bearer,
        refresh_token: None,
        expires_in: 3600,
        id_token: Some(id_token),
    };
    
    debug!("token response: {:?}", token);
    
    Ok(Response::with((status::Ok, try!(serde_json::to_string(&token).map_err(OpenIdConnectError::from)))))
}

