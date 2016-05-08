use iron::prelude::*;
use iron::status;
use bodyparser;

use config::Config;
use result::*;
use helpers::*;
use oauth2::*;

use serde_json;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientApplicationList {
    items: Vec<ClientApplication>
}

impl ClientApplicationList {
    pub fn new(apps: Vec<ClientApplication>) -> ClientApplicationList {
        ClientApplicationList {
            items: apps,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientApplicationUpdate {
    name: Option<String>,
    redirect_uris: Option<Vec<String>>,
}

impl ClientApplicationUpdate {
    pub fn apply(self, client_app: &mut ClientApplication) {
        client_app.name = self.name;
        client_app.redirect_uris = self.redirect_uris.unwrap_or(vec![]);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientApplicationView {
    pub name: Option<String>,
    pub client_id: String,
    pub redirect_uris: Vec<String>,
}

impl ClientApplicationView {
    pub fn new(client_app: ClientApplication) -> ClientApplicationView {
        ClientApplicationView {
            name: client_app.name,
            client_id: client_app.client_id,
            redirect_uris: client_app.redirect_uris,
        }
    }
}

pub fn applications_get_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    
    let apps_list = ClientApplicationList::new(try!(config.application_repo.get_client_applications()));
    
    let apps_json = try!(serde_json::to_string(&apps_list).map_err(OpenIdConnectError::from));
    
    Ok(Response::with((status::Ok, apps_json)))
}

pub fn read_client_application_body(req: &mut Request) -> Result<ClientApplication> {
    let maybe_json = try!(req.get::<bodyparser::Raw>());
    
    let json = try!(maybe_json.ok_or(OpenIdConnectError::EmptyPostBody));
    
    serde_json::from_str(&json).map_err(OpenIdConnectError::from)
}

pub fn applications_post_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    
    let ca = try!(config.application_repo.create_client_application(ClientApplicationBuilder::new()));
    
    let ca_json: String = try!(serde_json::to_string(&ca).map_err(OpenIdConnectError::from));
    
    Ok(Response::with((status::Ok, ca_json)))
}

pub fn applications_put_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    
    let ref client_id = try!(get_url_param(req, "id"));
    
    let maybe_update = try!(req.get::<bodyparser::Struct<ClientApplicationUpdate>>().map_err(OpenIdConnectError::from));
    let update = try!(maybe_update.ok_or(OpenIdConnectError::EmptyPostBody));
   
    let maybe_client_app = try!(config.application_repo.find_client_application(client_id));
    let mut client_app = try!(maybe_client_app.ok_or(OpenIdConnectError::ClientApplicationNotFound));
    
    update.apply(&mut client_app);  
    try!(config.application_repo.update_client_application(&client_app));
    
    let client_app_view = ClientApplicationView::new(client_app); 
    let update_json: String = try!(serde_json::to_string(&client_app_view).map_err(OpenIdConnectError::from));
    
    Ok(Response::with((status::Ok, update_json)))
}

pub fn applications_delete_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    
    let ref client_id = try!(get_url_param(req, "id"));
    
    try!(config.application_repo.remove_client_application(client_id));
    
    Ok(Response::with((status::Ok, "")))
}