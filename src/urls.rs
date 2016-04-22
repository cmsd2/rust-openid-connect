use std::collections::HashMap;
use iron::prelude::*;
use iron;
use url;

use result::{Result, OpenIdConnectError};

pub fn relative_url(_req: &mut Request, s: &str, maybe_params: Option<HashMap<String, String>>) -> Result<iron::Url> {
    //TODO use headers to figure out actual hostname
     
    let absolute = format!("http://localhost:3000{}", s);
        
    let mut uri = try!(url::Url::parse(&absolute).map_err(OpenIdConnectError::from));
    
    //let mut query_pairs = vec![];
    
    if let Some(params) = maybe_params {
        uri.set_query_from_pairs(params);
    }
    
    iron::Url::from_generic_url(uri).map_err(|e| OpenIdConnectError::UrlError(e))
}