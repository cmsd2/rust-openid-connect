use std::collections::HashMap;
use iron::prelude::*;
use iron;
use result::*;
use urlencoded::UrlEncodedQuery;
use urls::*;
use jsonwebtoken::jwt::{Jwt, JwtClaims};
use jsonwebtoken::JsonValueMapAccessors;
use jsonwebtoken::claims::claims_verifier;
use config::Config;
use chrono::*;
use serde::Serialize;
use serde_json;
use rbvt::params::*;
use rbvt::validation::{ValidationSchema, Validator};
use rbvt::state::*;

pub type RedirectToken = Jwt;

pub trait RedirectTokenConstructors {
    fn new_for_path(path: &str) -> RedirectToken;
    fn new_for_path_and_params(path: &str, params: &HashMap<String, String>) -> RedirectToken;
}

impl RedirectTokenConstructors for RedirectToken {
    fn new_for_path(path: &str) -> RedirectToken {
        let now = UTC::now().timestamp();
        let later = now + 3600;
        
        let mut t = RedirectToken::new();
        t.header.typ = Some("redirect".to_owned());
        t.claims.set_value("nbf", &now);
        t.claims.set_value("exp", &later);
        t.claims.set_value("path", &path.to_owned());
        t
    }
    
    fn new_for_path_and_params(path: &str, params: &HashMap<String, String>) -> RedirectToken {
        let mut t = Self::new_for_path(path);
        
        t.claims.set_value("params", params);
        
        t
    }
}

pub fn redirect_back(req: &mut Request, params: &HashMap<String, Vec<String>>) -> Result<Option<iron::Url>> {
    let config = try!(Config::get(req));
    let return_str = try!(multimap_get_maybe_one(params, "return"));
    
    if let Some(return_str) = return_str {
        let token = try!(Jwt::decode(&return_str, &config.mac_signer));
        
        match token.header.typ.as_ref().map(|s| &s[..]) {
            Some("authorize") => Ok(Some(try!(relative_url(req, "/authorize", Some(return_params(&return_str)))).to_owned())),
            Some("redirect") => {
                let mut v = claims_verifier::<JwtClaims>();
                let valid = try!(v.validate(&token.claims));
                if valid {
                    let params = try!(token.claims.get_value::<HashMap<String, String>>("params"));
                    
                    Ok(Some(try!(relative_url(req, "/authorize", params)) ))
                } else {
                    Err(OpenIdConnectError::RoutingError(format!("redirect token is not valid: {:?}", v.state)))
                }
            },
            Some(other) => Err(OpenIdConnectError::RoutingError(format!("no route for {}", other))),
            None => Err(OpenIdConnectError::RoutingError(format!("can't route unknown token"))),
        }
    } else {
        Ok(None)
    }
}

pub fn return_params(return_state: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    
    params.insert("jwt_req".to_owned(), return_state.to_owned());
    
    params
}