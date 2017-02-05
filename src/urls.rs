use std::collections::HashMap;
use iron::prelude::*;
use iron;
use iron::modifier::Modifier;
use iron::headers;
use url;

use result::{Result, OpenIdConnectError};
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
    
    let mut u = req.url.clone().into_generic_url();
    
    if let Some(override_url) = site_config.base_url.as_ref() {
        u.set_scheme(override_url.url.scheme());
        u.set_host(override_url.url.clone().into_generic_url().host_str());
        u.set_port(Some(override_url.url.port()));
    }
    
    if site_config.use_x_forwarded_proto {
        if let Some(proto) = try!(get_forwarded_proto(req)) {
            u.set_scheme(proto);
        }
    }
    
    if site_config.use_x_forwarded_port {
        if let Some(port) = try!(get_forwarded_port(req)) {
            u.set_port(Some(port));
        }
    }
    
    iron::Url::from_generic_url(u).map_err(|e| OpenIdConnectError::UrlError(e))
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

pub fn serialize_query_pairs(params: HashMap<String, Vec<String>>) -> String {
    let serializer = url::form_urlencoded::Serializer::new(String::new());
    serializer.extend_pairs(params.to_query_pairs());
    serializer.finish()
}

pub fn serialize_query_pairs_vec(params: Vec<(String, String)>) -> String {
    let serializer = url::form_urlencoded::Serializer::new(String::new());
    serializer.extend_pairs(params);
    serializer.finish()
}

pub fn relative_url(req: &mut Request, s: &str, maybe_params: Option<HashMap<String, Vec<String>>>) -> Result<iron::Url> {
    //TODO use headers to figure out actual hostname
    //let forwarded_for = req.headers.get()
    debug!("headers: {:?}", req.headers);
    
    let mut uri = try!(get_absolute_url(req)).into_generic_url();
    
    debug!("currently at {:?}", uri);
    debug!("rel url for {} {:?}", s, maybe_params);
    
    let path: String = s.split("/")
        .map(|part| part.to_owned())
        .filter(|part| part.len() > 0)
        .collect();

    uri.set_path(&path);
    uri.set_query(maybe_params.map(serialize_query_pairs).map(|s| &s[..]));
    uri.set_fragment(None);
    
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
