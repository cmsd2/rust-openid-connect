use std::collections::HashMap;
use iron::prelude::*;
use iron;
use url;

use result::{Result, OpenIdConnectError};

pub fn relative_url(req: &mut Request, s: &str, maybe_params: Option<HashMap<String, Vec<String>>>) -> Result<iron::Url> {
    //TODO use headers to figure out actual hostname
    //let forwarded_for = req.headers.get()
    debug!("headers: {:?}", req.headers);
    
    let absolute = format!("http://localhost:3000{}", s);
        
    let mut uri = try!(url::Url::parse(&absolute).map_err(OpenIdConnectError::from));
    
    if let Some(params) = maybe_params {
        let mut query_pairs = vec![];
        for (k,vs) in params {
            for v in vs {
                query_pairs.push((k.clone(), v));
            }
        }
    
        uri.set_query_from_pairs(query_pairs);
    }
    
    iron::Url::from_generic_url(uri).map_err(|e| OpenIdConnectError::UrlError(e))
}