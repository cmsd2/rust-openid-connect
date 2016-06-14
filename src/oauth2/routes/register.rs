use std::collections::HashMap;

use iron::prelude::*;
use iron::status;
use iron::mime::Mime;
use serde_json;
use urlencoded::UrlEncodedBody;

use result::*;
use site_config::*;
use config::*;
use oauth2::models::registration::*;
use authentication::*;

pub fn register_client_application(req: &mut Request, params: HashMap<String, Vec<String>>) -> Result<RegistrationResult> {
    let config = try!(Config::get(req));

    let mut reg_req = RegistrationRequest::new();
    try!(reg_req.load_params(&params));
    
    let secret = Some(new_secret());
    let mut client_builder = reg_req.to_client_builder();
    client_builder.secret = secret.clone();

    let client = try!(config.application_repo.create_client_application(client_builder));

    Ok(RegistrationResult::from_client(client, secret))
}

/*
example request:
DEBUG:openid_connect::oauth2::routes::register: dynamic application registration request body: {"{\"response_types\": [\
"code\"], \"redirect_uris\": [\"oob://localhost/callback\"], \"application_type\": \"web\"}": [""]}
*/

pub fn register_application_post_handler(req: &mut Request) -> IronResult<Response> {
    debug!("/connect/register");
    let config = try!(Config::get(req));
    let site_config = try!(SiteConfig::get(req));
    let hashmap = try!(req.get::<UrlEncodedBody>().map_err(OpenIdConnectError::from));
    debug!("dynamic application registration request body: {:?}", hashmap);
    let reg_result = try!(register_client_application(req, hashmap));
    
    let reg_result_json = try!(serde_json::to_string(&reg_result).map_err(OpenIdConnectError::from));
    let content_type = "application/json".parse::<Mime>().unwrap();

    Ok(Response::with((content_type, status::Ok, reg_result_json)))
}
