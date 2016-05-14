use std::collections::HashMap;
use iron::prelude::*;
use iron;
use iron::modifier::Modifier;
use iron::headers;
use url;

use result::Result;
use site_config::*;
use x_headers::*;

pub fn get_forwarded_proto<'a>(req: &'a mut Request) -> Result<Option<&'a str>> {
    let maybe_proto: Option<&XForwardedProto> = req.headers.get::<XForwardedProto>();
    
    Ok(maybe_proto.map(|p| &p.forwarded_proto[..]))
}

pub fn get_forwarded_port(req: &mut Request) -> Result<Option<u16>> {
    let maybe_port: Option<&XForwardedPort> = req.headers.get::<XForwardedPort>();
    
    Ok(maybe_port.map(|p| p.forwarded_port))
}

pub fn get_absolute_url(req: &mut Request) -> Result<iron::Url> {
    let site_config = try!(SiteConfig::get(req));
    
    let mut u = req.url.clone();
    
    if let Some(override_url) = site_config.base_url.as_ref() {
        u.scheme = override_url.url.scheme.clone();
        u.host = override_url.url.host.clone();
        u.port = override_url.url.port;
    }
    
    if site_config.use_x_forwarded_proto {
        if let Some(proto) = try!(get_forwarded_proto(req)) {
            u.scheme = proto.to_owned();
        }
    }
    
    if site_config.use_x_forwarded_port {
        if let Some(port) = try!(get_forwarded_port(req)) {
            u.port = port;
        }
    }
    
    Ok(u)
}

pub trait ToQueryPairs {
    fn to_query_pairs(&self) -> Vec<(String, String)>;
}

impl ToQueryPairs for HashMap<String, Vec<String>> {
    fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut query_pairs = vec![];

        for (k,vs) in self {
            for v in vs {
                query_pairs.push((k.clone(), v.clone()));
            }
        }
        
        query_pairs
    }
}

pub fn relative_url(req: &mut Request, s: &str, maybe_params: Option<HashMap<String, Vec<String>>>) -> Result<iron::Url> {
    //TODO use headers to figure out actual hostname
    //let forwarded_for = req.headers.get()
    debug!("headers: {:?}", req.headers);
    
    let mut uri = try!(get_absolute_url(req));
    
    debug!("currently at {:?}", uri);
    debug!("rel url for {} {:?}", s, maybe_params);
    
    uri.path = s.split("/").map(|part| part.to_owned()).filter(|part| part.len() > 0).collect();
    uri.query = maybe_params.map(|p| url::form_urlencoded::serialize(p.to_query_pairs()));
    uri.fragment = None;
    
    Ok(uri)
}

// replacement for iron's which has a private inner field
pub struct RoidcRedirectRaw(pub String);

impl Modifier<Response> for RoidcRedirectRaw {
    fn modify(self, res: &mut Response) {
        let RoidcRedirectRaw(path) = self;
        res.headers.set(headers::Location(path));
    }
}
