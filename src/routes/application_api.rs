use iron::prelude::*;
use iron::status;
use bodyparser;

use config::Config;
use result::*;
use client_application::*;

use serde_json;

#[derive(Serialize, Deserialize)]
pub struct ClientApplications {
    items: Vec<ClientApplication>
}

impl ClientApplications {
    pub fn new(apps: Vec<ClientApplication>) -> ClientApplications {
        ClientApplications {
            items: apps,
        }
    }
}

pub fn applications_get_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
    let apps_list = ClientApplications::new(try!(config.application_repo.get_client_applications()));
    
    let apps_json = try!(serde_json::to_string(&apps_list).map_err(OpenIdConnectError::from));
    
    Ok(Response::with((status::Ok, apps_json)))
}

pub fn read_client_application_body(req: &mut Request) -> Result<ClientApplication> {
    let maybe_json = try!(req.get::<bodyparser::Raw>());
    
    let json = try!(maybe_json.ok_or(OpenIdConnectError::NotImplemented));
    
    serde_json::from_str(&json).map_err(OpenIdConnectError::from)
}

pub fn applications_post_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
    let ca = try!(config.application_repo.create_client_application());
    
    let ca_json: String = try!(serde_json::to_string(&ca).map_err(OpenIdConnectError::from));
    
    Ok(Response::with((status::Ok, ca_json)))
}

pub fn applications_put_handler(_config: &Config, req: &mut Request) -> IronResult<Response> {
    Err(IronError::from(OpenIdConnectError::NotImplemented))
}

pub fn applications_delete_handler(_config: &Config, req: &mut Request) -> IronResult<Response> {
    Err(IronError::from(OpenIdConnectError::NotImplemented))
}