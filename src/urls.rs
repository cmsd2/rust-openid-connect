use iron::prelude::*;
use iron::Url;

use result::{Result, OpenIdConnectError};

pub fn relative_url(_req: &mut Request, s: &str) -> Result<Url> {
    //TODO use headers to figure out actual hostname
     
    let absolute = format!("http://localhost:3000{}", s);
        
    Url::parse(&absolute).map_err(|e| OpenIdConnectError::UrlParseError(e.to_owned()))
}