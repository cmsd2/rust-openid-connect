use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;
use urlencoded::UrlEncodedBody;
use serde_json::value;
use rbvt::params::*;

use config::Config;
use result::*;
use oauth2::models::*;
use oauth2::repos::*;
use view::View;
use helpers::*;
use urls::relative_url;

pub fn applications_index_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    
    let apps_list = try!(config.application_repo.get_client_applications());
    
    let mut view = try!(View::new_for_session("applications/index.html", req));
    
    view.data.insert("applications".to_owned(), try!(value::to_value(&apps_list).map_err(OpenIdConnectError::from)));
    
    Ok(Response::with((status::Ok, try!(view.template().map_err(OpenIdConnectError::from)))))
}

pub fn applications_new_handler(req: &mut Request) -> IronResult<Response> {  
    let view = try!(View::new_for_session("applications/new.html", req));

    Ok(Response::with((status::Ok, try!(view.template().map_err(OpenIdConnectError::from)))))
}

pub fn applications_show_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    
    let ref client_id = try!(get_url_param(req, "id"));
    
    let maybe_client_app = try!(config.application_repo.find_client_application(client_id));
    let client_app = try!(maybe_client_app.ok_or(OpenIdConnectError::ClientApplicationNotFound));
    
    let mut view = try!(View::new_for_session("applications/show.html", req));
    
    view.data.insert("client_name".to_owned(), try!(value::to_value(&client_app.client_name).map_err(OpenIdConnectError::from)));
    view.data.insert("redirect_uris".to_owned(), try!(value::to_value(&client_app.redirect_uris).map_err(OpenIdConnectError::from)));
    view.data.insert("client_id".to_owned(), try!(value::to_value(&client_id).map_err(OpenIdConnectError::from)));
    
    Ok(Response::with((status::Ok, try!(view.template().map_err(OpenIdConnectError::from)))))
}

pub fn applications_edit_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    
    let ref client_id = try!(get_url_param(req, "id"));
    
    let maybe_client_app = try!(config.application_repo.find_client_application(client_id));
    let client_app = try!(maybe_client_app.ok_or(OpenIdConnectError::ClientApplicationNotFound));
    
    let mut view = try!(View::new_for_session("applications/edit.html", req));
    
    view.data.insert("client_name".to_owned(), try!(value::to_value(&client_app.client_name).map_err(OpenIdConnectError::from)));
    view.data.insert("redirect_uris".to_owned(), try!(value::to_value(&client_app.redirect_uris).map_err(OpenIdConnectError::from)));
    view.data.insert("client_id".to_owned(), try!(value::to_value(&client_id).map_err(OpenIdConnectError::from)));
    
    Ok(Response::with((status::Ok, try!(view.template().map_err(OpenIdConnectError::from)))))
}

pub fn applications_update_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    
    let ref client_id = try!(get_url_param(req, "id"));
    let show_redirect_url = try!(relative_url(req, "/applications", None));
    
    let params = try!(req.get_ref::<UrlEncodedBody>().map_err(OpenIdConnectError::from));

    if try!(multimap_get_maybe_one(params, "cancel").map_err(OpenIdConnectError::from)).is_none() {

        let mut builder = ClientApplicationBuilder::new();
        try!(builder.load_params(params));
    
        let maybe_client_app = try!(config.application_repo.find_client_application(client_id));
        let mut client_app = try!(maybe_client_app.ok_or(OpenIdConnectError::ClientApplicationNotFound));
    
        client_app.client_name = builder.client_name;
        client_app.redirect_uris = builder.redirect_uris.unwrap_or(vec![]);
    
        try!(config.application_repo.update_client_application(&client_app));
    }
    
    Ok(Response::with((status::Found, Redirect(show_redirect_url))))
}

pub fn applications_create_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    
    let params = try!(req.get_ref::<UrlEncodedBody>().map_err(OpenIdConnectError::from)).clone();
    
    if try!(multimap_get_maybe_one(&params, "cancel").map_err(OpenIdConnectError::from)).is_none() {
        let mut builder = ClientApplicationBuilder::new();
    
        try!(builder.load_params(&params));
    
        let ca = try!(config.application_repo.create_client_application(builder));

        let show_redirect_url = try!(relative_url(req, &format!("/applications/{}", ca.client_id), None));
    
        Ok(Response::with((status::Found, Redirect(show_redirect_url))))
    } else {
        let cancel_redirect_url = try!(relative_url(req, &format!("/applications"), None));
        
        Ok(Response::with((status::Found, Redirect(cancel_redirect_url))))
    }
}
/*
pub fn read_client_application_body(req: &mut Request) -> Result<ClientApplication> {
    let maybe_json = try!(req.get::<bodyparser::Raw>());
    
    let json = try!(maybe_json.ok_or(OpenIdConnectError::EmptyPostBody));
    
    serde_json::from_str(&json).map_err(OpenIdConnectError::from)
}

pub fn applications_post_handler(config: &Config, _req: &mut Request) -> IronResult<Response> {
    let ca = try!(config.application_repo.create_client_application());
    
    let ca_json: String = try!(serde_json::to_string(&ca).map_err(OpenIdConnectError::from));
    
    Ok(Response::with((status::Ok, ca_json)))
}

pub fn get_url_param(req: &mut Request, name: &str) -> Result<String> {
    let params = try!(req.extensions.get::<Router>().ok_or(params::ParamError::NotFound("id".to_owned())));
    
    let value = try!(params.find(name).map(|s| s.to_owned()).ok_or(params::ParamError::NotFound("id".to_owned())));
    
    Ok(value)
}

pub fn applications_put_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
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

pub fn applications_delete_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
    let ref client_id = try!(get_url_param(req, "id"));
    
    try!(config.application_repo.remove_client_application(client_id));
    
    Ok(Response::with((status::Ok, "")))
}
*/