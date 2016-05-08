use std::collections::HashMap;
use iron::prelude::*;
use iron;
use result::*;
use urlencoded::UrlEncodedQuery;
use urls::*;
use jsonwebtoken::jwt::Jwt;
use validation::params::*;
use config::Config;

pub fn redirect_back(req: &mut Request, params: &HashMap<String, Vec<String>>) -> Result<Option<iron::Url>> {
    let config = try!(Config::get(req));
    let return_str = try!(multimap_get_maybe_one(params, "return"));
    
    if let Some(return_str) = return_str {
        let token = try!(Jwt::decode(&return_str, &config.mac_signer));
        
        match token.header.typ.as_ref().map(|s| &s[..]) {
            Some("authorize") => Ok(Some(try!(relative_url(req, "/authorize", Some(return_params(&return_str)))).to_owned())),
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