use std::collections::HashMap;
use plugin::Plugin as PluginPlugin;
use iron;
use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;
use urlencoded::UrlEncodedBody;
use serde_json::value;

use config::Config;
use result::*;
use oauth2::*;
use view::View;
use helpers::*;
use urls::relative_url;
use sessions::*;
use back::*;

pub fn grants_redirect_url(req: &mut Request, grants_path: &str, path: &str) -> Result<iron::Url> {
    redirect_forwards_url(req, grants_path, path, HashMap::new())
}

pub fn grants_index_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    let session = try!(UserSession::eval(req));
    let authenticated = session.as_ref().map(|s| s.authenticated).unwrap_or(false);
    let login_url = try!(grants_redirect_url(req, "/grants", "/login"));
    
    if ! authenticated {
        return Ok(Response::with((status::Found, Redirect(login_url))))
    }
    
    let user_session = try!(session.ok_or(OpenIdConnectError::NoSessionLoaded));
    let user_id = try!(user_session.user_id.ok_or(OpenIdConnectError::UserNotFound));
    
    let mut view = try!(View::new_for_session("grants/index.html", req));
   
    let grants_list = try!(config.grant_repo.get_user_grants(&user_id));
    debug!("found grants: {:?}", grants_list);
    view.data.insert("grants".to_owned(), value::to_value(&grants_list));
    debug!("rendering view {:?}", view);
    
    Ok(Response::with((status::Ok, view.template())))
}

pub fn grants_show_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    let session = try!(UserSession::eval(req));
    let authenticated = session.as_ref().map(|s| s.authenticated).unwrap_or(false);
    let ref client_id = try!(get_url_param(req, "id"));
    let login_url = try!(grants_redirect_url(req, "/grants", "/login")); // fix return url
    
    if ! authenticated {
        return Ok(Response::with((status::Found, Redirect(login_url))))
    }
    
    let user_session = try!(session.ok_or(OpenIdConnectError::NoSessionLoaded));
    let user_id = try!(user_session.user_id.ok_or(OpenIdConnectError::UserNotFound));
    
    let maybe_client_app = try!(config.application_repo.find_client_application(client_id));
    let client_app = try!(maybe_client_app.ok_or(OpenIdConnectError::ClientApplicationNotFound));
    let maybe_grant = try!(config.grant_repo.find_grant(&user_id, client_id));
    let grant = try!(maybe_grant.ok_or(OpenIdConnectError::GrantNotFound));
    
    let mut view = try!(View::new_for_session("grants/show.html", req));
    
    view.data.insert("name".to_owned(), value::to_value(&client_app.name));
    view.data.insert("permissions_allowed".to_owned(), value::to_value(&grant.permissions_allowed));
    view.data.insert("permissions_denied".to_owned(), value::to_value(&grant.permissions_denied));
    view.data.insert("client_id".to_owned(), value::to_value(&client_id));
    
    Ok(Response::with((status::Ok, view.template())))
}

pub fn grants_edit_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    let session = try!(UserSession::eval(req));
    let authenticated = session.as_ref().map(|s| s.authenticated).unwrap_or(false);
    let ref client_id = try!(get_url_param(req, "id"));
    let login_url = try!(grants_redirect_url(req, "/grants", "/login")); // fix return url
    
    if ! authenticated {
        return Ok(Response::with((status::Found, Redirect(login_url))))
    }
    
    let user_session = try!(session.ok_or(OpenIdConnectError::NoSessionLoaded));
    let user_id = try!(user_session.user_id.ok_or(OpenIdConnectError::UserNotFound));
    
    let maybe_client_app = try!(config.application_repo.find_client_application(client_id));
    let client_app = try!(maybe_client_app.ok_or(OpenIdConnectError::ClientApplicationNotFound));
    let maybe_grant = try!(config.grant_repo.find_grant(&user_id, client_id));
    let grant = try!(maybe_grant.ok_or(OpenIdConnectError::GrantNotFound));
    
    let mut view = try!(View::new_for_session("grants/edit.html", req));
    
    view.data.insert("name".to_owned(), value::to_value(&client_app.name));
    view.data.insert("permissions_allowed".to_owned(), value::to_value(&grant.permissions_allowed));
    view.data.insert("permissions_denied".to_owned(), value::to_value(&grant.permissions_denied));
    view.data.insert("client_id".to_owned(), value::to_value(&client_id));
    
    Ok(Response::with((status::Ok, view.template())))
}

pub fn grants_update_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    let session = try!(UserSession::eval(req));
    let authenticated = session.as_ref().map(|s| s.authenticated).unwrap_or(false);
    let ref client_id = try!(get_url_param(req, "id"));
    let login_url = try!(grants_redirect_url(req, "/grants", "/login")); //TODO fix return url
    let show_redirect_url = try!(relative_url(req, "/grants", None));
    
    if ! authenticated {
        return Ok(Response::with((status::Found, Redirect(login_url))))
    }
    
    let user_session = try!(session.ok_or(OpenIdConnectError::NoSessionLoaded));
    let user_id = try!(user_session.user_id.ok_or(OpenIdConnectError::UserNotFound));
    
    let maybe_client_app = try!(config.application_repo.find_client_application(client_id));
    let client_app = try!(maybe_client_app.ok_or(OpenIdConnectError::ClientApplicationNotFound));
    let maybe_grant = try!(config.grant_repo.find_grant(&user_id, client_id));
    let grant = try!(maybe_grant.ok_or(OpenIdConnectError::GrantNotFound));
    
    //TODO update grant
    
    Ok(Response::with((status::Found, Redirect(show_redirect_url))))
}
