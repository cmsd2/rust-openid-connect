use std::collections::HashMap;

use iron::prelude::*;
use iron::status;
use urlencoded::UrlEncodedBody;

use result::{Result, OpenIdConnectError};
use vlad::params::*;
use config::Config;
use vlad::result;
use vlad::result::VladError;
use vlad::state::*;
use vlad::builder::*;

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
                        .ok_or(OpenIdConnectError::from(VladError::MissingRequiredValue("grant_type".to_owned())))
                        .and_then(|gt| GrantType::from_str(&gt))),
                code: self.code,
                redirect_uri: try!(self.redirect_uri.ok_or(VladError::MissingRequiredValue("redirect_uri".to_owned())))
            })
        } else {
            Err(OpenIdConnectError::from(result::VladError::ValidationError(self.validation_state)))
        }
    }
    
    pub fn load_params(&mut self, params: &HashMap<String, Vec<String>>) -> Result<()> {
        let mut grant_type: Option<GrantType> = None;
        
        if let Some(grant_type_str) = try!(multimap_get_maybe_one(params, "grant_type")) {
            let my_grant_type_str = grant_type_str.to_owned();
            if let Ok(a_grant_type) = GrantType::from_str(&my_grant_type_str) {
                grant_type = Some(a_grant_type);
                self.grant_type = Some(my_grant_type_str);
            } else {
                self.validation_state.reject("grant_type", result::VladError::InvalidValue("grant_type".to_owned()));
            }
        } else {
            self.validation_state.reject("grant_type", result::VladError::MissingRequiredValue("grant_type".to_owned()));
        }
        
        if let Some(code) = try!(multimap_get_maybe_one(params, "code")) {
            self.code = Some(code.to_owned());
        } else if grant_type == Some(GrantType::AuthorizationCode) {
            self.validation_state.reject("code", result::VladError::MissingRequiredValue("code".to_owned()));
        }
            
        Ok(())
    }
    
    pub fn build_from_params(hashmap: &HashMap<String, Vec<String>>) -> Result<TokenRequest> {
        let mut builder = TokenRequestBuilder::new();

        try!(builder.load_params(hashmap));
        
        builder.build()
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

impl TokenRequest {
    pub fn new(grant_type: GrantType, code: Option<String>, redirect_uri: String) -> TokenRequest {
        TokenRequest {
            grant_type: grant_type,
            code: code,
            redirect_uri: redirect_uri,
        }
    }
    
    pub fn from_params(hashmap: &HashMap<String, Vec<String>>) -> Result<TokenRequest> {
        TokenRequestBuilder::build_from_params(hashmap)
    }
}

pub fn parse_token_request(req: &mut Request) -> Result<TokenRequest> {
    let hashmap = try!(req.get_ref::<UrlEncodedBody>());
    debug!("token request body: {:?}", hashmap);
    
    //TODO validate supplied oauth2 params
    
    let token_request = try!(TokenRequest::from_params(hashmap));
    
    Ok(token_request)
}

pub fn token_post_handler(_config: &Config, req: &mut Request) -> IronResult<Response> {
    debug!("/token");
    
    let token_request = try!(parse_token_request(req));
    
    debug!("token request: {:?}", token_request);
    
    Ok(Response::with((status::Ok, "insert id_token here")))
}

