use std::collections::HashMap;

use rbvt::result::{ValidationError};
use rbvt::validation::*;
use rbvt::state::*;
use rbvt::params::*;

use result::*;

pub static OIDC_V1_ISSUER: &'static str = "http://openid.net/specs/connect/1.0/issuer";


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WebFingerLink  {
    pub rel: String,
    pub href: String,
}

impl WebFingerLink {
    pub fn new<R,H>(rel: R, href: H) -> WebFingerLink where R: Into<String>, H: Into<String> {
        WebFingerLink {
            rel: rel.into(),
            href: href.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WebFingerResponse {
    pub subject: String,
    pub links: Vec<WebFingerLink>,
}

impl WebFingerResponse {
    pub fn new<S>(subject: S, links: Vec<WebFingerLink>) -> WebFingerResponse where S: Into<String> {
        WebFingerResponse {
            subject: subject.into(),
            links: links,
        }
    }
}

pub struct WebFingerRequestBuilder {
    pub resource: Option<String>,
    pub rel: Option<String>,
}

pub struct WebFingerRequest {
    pub resource: String,
    pub rel: String,
}

impl WebFingerRequestBuilder {
    pub fn new() -> WebFingerRequestBuilder {
        WebFingerRequestBuilder {
            resource: None,
            rel: None,
        }
    }
    
    pub fn from_params(params: &HashMap<String, Vec<String>>) -> Result<WebFingerRequestBuilder> {
        let mut wf = WebFingerRequestBuilder::new();
        wf.resource = try!(multimap_get_maybe_one(params, "resource")).map(|s| s.to_owned());
        wf.rel = try!(multimap_get_maybe_one(params, "rel")).map(|s| s.to_owned());
        
        let mut vs = ValidationSchema::new();
        
        vs.rule(Box::new(|b: &WebFingerRequestBuilder, s: &mut ValidationState| {
            if b.resource.is_none() {
                s.reject("resource", ValidationError::MissingRequiredValue("resource".to_owned()));
            }
            Ok(())
        }));
        
        vs.rule(Box::new(|b: &WebFingerRequestBuilder, s: &mut ValidationState| {
            if let Some(ref rel) = b.rel {
                if rel != "http://openid.net/specs/connect/1.0/issuer" {
                    s.reject("rel", ValidationError::InvalidValue("rel".to_owned()));
                }
            } else {
                s.reject("rel", ValidationError::MissingRequiredValue("rel".to_owned()));
            }
            Ok(())
        }));
        
        if ! try!(vs.validate(&wf)) {
            Err(OpenIdConnectError::ValidationError(ValidationError::ValidationError(vs.state)))
        } else {
            Ok(wf)
        }
    }
    
    pub fn build(self) -> Result<WebFingerRequest> {
        Ok(WebFingerRequest {
            resource: try!(self.resource.ok_or(OpenIdConnectError::ValidationError(ValidationError::MissingRequiredValue("resource".to_owned())))),
            rel: try!(self.rel.ok_or(OpenIdConnectError::ValidationError(ValidationError::MissingRequiredValue("rel".to_owned())))),
        })
    }
}
