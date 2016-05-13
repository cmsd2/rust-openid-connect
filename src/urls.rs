use std::collections::HashMap;
use iron::prelude::*;
use iron;
use iron::modifier::Modifier;
use iron::headers;
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
        //let mut query_pairs = uri.query_pairs_mut();
        
        for (k,vs) in params {
            for v in vs {
                query_pairs.push((k.clone(), v));
                // query_pairs.append_pair(&k, &v);
            }
        }
        
        uri.set_query_from_pairs(query_pairs);
    }
    
    iron::Url::from_generic_url(uri).map_err(|e| OpenIdConnectError::UrlError(e))
}

// replacement for iron's which has a private inner field
pub struct RoidcRedirectRaw(pub String);

impl Modifier<Response> for RoidcRedirectRaw {
    fn modify(self, res: &mut Response) {
        let RoidcRedirectRaw(path) = self;
        res.headers.set(headers::Location(path));
    }
}
