use iron::prelude::*;
use iron::status;
use iron::mime::Mime;
use serde_json;
use urlencoded::UrlEncodedQuery;

use result::*;
use site_config::*;
use oauth2::models::webfinger_request::*;

/// /.well-known/webfinger
pub fn webfinger_get_handler(req: &mut Request) -> IronResult<Response> {
    let params = try!(req.get::<UrlEncodedQuery>().map_err(OpenIdConnectError::from));
    let wfb = try!(WebFingerRequestBuilder::from_params(&params));
    let wf = try!(wfb.build());
    let site_config = try!(SiteConfig::get(req));
    let issuer = try!(site_config.token_issuer.as_ref().ok_or(OpenIdConnectError::ConfigError("no issuer set".to_owned())));
    let link = WebFingerLink::new(OIDC_V1_ISSUER, &issuer[..]);
    
    // TODO validate subject
    let webfinger_result = WebFingerResponse::new(wf.resource, vec![link]);
    
    let content_type = "application/json".parse::<Mime>().unwrap();
    
    let body_json = try!(serde_json::to_string(&webfinger_result).map_err(OpenIdConnectError::from));
    
    Ok(Response::with((content_type, status::Ok, body_json)))
}