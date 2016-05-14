use std::collections::HashMap;
use iron::prelude::*;
use iron;
use result::*;
use urls::*;
use jsonwebtoken::jwt::{Jwt, JwtClaims};
use jsonwebtoken::json::*;
use jsonwebtoken::claims::claims_verifier;
use config::Config;
use chrono::*;
use rbvt::params::*;
use rbvt::validation::Validator;

pub type RedirectToken = Jwt;

pub trait RedirectTokenConstructors {
    fn new_for_path(path: &str) -> RedirectToken;
    fn new_for_path_and_params(path: &str, params: &HashMap<String, Vec<String>>) -> RedirectToken;
}

impl RedirectTokenConstructors for RedirectToken {
    fn new_for_path(path: &str) -> RedirectToken {
        let now = UTC::now().timestamp();
        let later = now + 3600;
        
        let mut t = RedirectToken::default();
        t.header.typ = Some("redirect".to_owned());
        t.claims.set_value("nbf", &now);
        t.claims.set_value("exp", &later);
        t.claims.set_value("path", &path.to_owned());
        t
    }
    
    fn new_for_path_and_params(path: &str, params: &HashMap<String, Vec<String>>) -> RedirectToken {
        let mut t = Self::new_for_path(path);
        
        t.claims.set_value("params", params);
        
        t
    }
}

pub fn redirect_forwards_url(req: &mut Request, return_path: &str, redirect_path: &str, return_payload: HashMap<String, Vec<String>>) -> Result<iron::Url> {
    let config = try!(Config::get(req));
    
    let mut params = HashMap::new();
    
    let redirect_token = RedirectToken::new_for_path_and_params(return_path, &return_payload);
    
    params.insert("return".to_owned(), vec![try!(redirect_token.encode(&config.mac_signer))]);

    relative_url(req, redirect_path, Some(params))
}

pub fn load_token(req: &mut Request, params: &HashMap<String, Vec<String>>, token_param_name: &str) -> Result<Option<Jwt>> {
    let config = try!(Config::get(req));
    let return_str = try!(multimap_get_maybe_one(params, token_param_name));
    
    if let Some(return_str) = return_str {
        let token = try!(Jwt::decode(&return_str, &config.mac_signer));
        
        let mut v = claims_verifier::<JwtClaims>();
        let valid = try!(v.validate(&token.claims));
        if valid {
            Ok(Some(token))
        } else {
            Err(OpenIdConnectError::RoutingError(format!("token is not valid: {:?}", v.state)))
        }
    } else {
        Ok(None)
    }
}

pub fn redirect_back_url(req: &mut Request, params: &HashMap<String, Vec<String>>) -> Result<Option<iron::Url>> {
    if let Some(token) = try!(load_token(req, params, "return")) {
    
        match token.header.typ.as_ref().map(|s| &s[..]) {
            Some("redirect") => {
                let params = try!(token.claims.get_value::<HashMap<String, Vec<String>>>("params"));
                let maybe_path = try!(token.claims.get_value::<String>("path"));
                let path = try!(maybe_path.ok_or(OpenIdConnectError::RoutingError("redirect path not supplied in token claims".to_owned())));
        
                Ok(Some(try!(relative_url(req, &path, params)) ))
            },
            Some(other) => Err(OpenIdConnectError::RoutingError(format!("no route for {}", other))),
            None => Err(OpenIdConnectError::RoutingError(format!("can't route unknown token"))),
        }
    } else {
        Ok(None)
    }
}

pub fn return_params<S: Into<String>>(return_state: S) -> HashMap<String, Vec<String>> {
    let mut params = HashMap::new();
    
    params.insert("request".to_owned(), vec![return_state.into()]);
    
    params
}