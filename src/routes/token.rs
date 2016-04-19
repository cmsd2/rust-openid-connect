use std::collections::HashMap;

use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;
use urlencoded::UrlEncodedQuery;

use result::{Result, OpenIdConnectError};
use params::*;
use urls::*;
use ::ResponseType;
use config::Config;

#[derive(Copy, Clone, Debug)]
pub enum GrantType {
    AuthorizationCode,
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


pub fn token_post_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
    debug!("/token");
    
    Ok(Response::with((status::Ok, "insert id_token here")))
}

