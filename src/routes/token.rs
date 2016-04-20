use std::collections::HashMap;

use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;
use urlencoded::UrlEncodedBody;

use result::{Result, OpenIdConnectError};
use params::*;
use urls::*;
use ::ResponseType;
use config::Config;

#[derive(Copy, Clone, Debug)]
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
    pub fn from_params(hashmap: &HashMap<String, Vec<String>>) -> Result<TokenRequest> {
        let grant_type = try!(multimap_get_one(hashmap, "grant_type"));
        let code = try!(multimap_get_maybe_one(hashmap, "code"));
        let redirect_uri = try!(multimap_get_one(hashmap, "redirect_uri"));
    
        Ok(TokenRequest {
            grant_type: try!(GrantType::from_str(grant_type)),
            code: code.map(|s| s.to_owned()),
            redirect_uri: redirect_uri.to_owned(),
        })
    }
}

pub fn parse_token_request(req: &mut Request) -> Result<TokenRequest> {
    let hashmap = try!(req.get_ref::<UrlEncodedBody>());
    debug!("token request body: {:?}", hashmap);
    
    //TODO validate supplied oauth2 params
    
    let token_request = try!(TokenRequest::from_params(hashmap));
    
    Ok(token_request)
}

pub fn token_post_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
    debug!("/token");
    
    let token_request = try!(parse_token_request(req));
    
    debug!("token request: {:?}", token_request);
    
    Ok(Response::with((status::Ok, "insert id_token here")))
}

