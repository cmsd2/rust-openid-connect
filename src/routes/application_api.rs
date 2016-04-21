use iron::prelude::*;
use iron::status;

use config::Config;
use result::*;
use client_application::*;

use rustc_serialize::json;

#[derive(RustcDecodable, RustcEncodable)]
pub struct ClientApplications {
    items: Vec<ClientApplication>
}

pub fn applications_get_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
    let apps = try!(config.application_repo.get_client_applications());
    
    let apps_list = ClientApplications {
        items: apps
    };
    
    let apps_json = try!(json::encode(&apps_list).map_err(|e| OpenIdConnectError::from(e)));
    
    Ok(Response::with((status::Ok, apps_json)))
}

pub fn applications_post_handler(_config: &Config, req: &mut Request) -> IronResult<Response> {
    Err(IronError::from(OpenIdConnectError::NotImplemented))
}

pub fn applications_put_handler(_config: &Config, req: &mut Request) -> IronResult<Response> {
    Err(IronError::from(OpenIdConnectError::NotImplemented))
}

pub fn applications_delete_handler(_config: &Config, req: &mut Request) -> IronResult<Response> {
    Err(IronError::from(OpenIdConnectError::NotImplemented))
}