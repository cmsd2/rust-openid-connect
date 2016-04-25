use std::collections::HashMap;

use iron::prelude::*;
use iron::status;
use urlencoded::UrlEncodedBody;

use result::{Result, OpenIdConnectError};
use config::Config;
use validation::params::*;
use validation::result;
use validation::result::ValidationError;
use validation::state::*;
use validation::builder::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GrantType {
    AuthorizationCode,
}

impl GrantType {
    pub fn from_str(s: &str) -> Result<GrantType> {
        match s {
            "authorization_code" => Ok(GrantType::AuthorizationCode),
            _ => Err(OpenIdConnectError::UnknownGrantType(Box::new(s.to_owned())))
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TokenType {
    Bearer,
}

#[derive(Clone, Debug)]
pub struct TokenRequest {
    grant_type: GrantType,
    code: Option<String>,
    redirect_uri: String,
}

impl TokenRequest {
    pub fn new(grant_type: GrantType, code: Option<String>, redirect_uri: String) -> TokenRequest {
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
                redirect_uri: try!(self.redirect_uri.ok_or(ValidationError::MissingRequiredValue("redirect_uri".to_owned())))
            })
        } else {
            Err(OpenIdConnectError::from(result::ValidationError::ValidationError(self.validation_state)))
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
                    self.validation_state.reject("code", result::ValidationError::MissingRequiredValue("code".to_owned()));
                }
            } else {
                self.validation_state.reject("grant_type", result::ValidationError::InvalidValue("grant_type".to_owned()));
            }
        } else {
            self.validation_state.reject("grant_type", result::ValidationError::MissingRequiredValue("grant_type".to_owned()));
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


pub fn token_post_handler(_config: &Config, req: &mut Request) -> IronResult<Response> {
    debug!("/token");
    
    let token_request = try!(TokenRequestBuilder::build_from_request(req));
    
    debug!("token request: {:?}", token_request);
    
    Ok(Response::with((status::Ok, "insert id_token here")))
}

