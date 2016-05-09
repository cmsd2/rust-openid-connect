use std::io::Read;
use std::collections::HashMap;
use std::borrow::Cow;

use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;
use urlencoded::*;
use serde_json::value;
use plugin::Plugin as PluginPlugin;

use rbvt::state::*;
use rbvt::result::ValidationError;
use rbvt::params::*;
use result::{Result, OpenIdConnectError};
use urls::*;
use config::Config;
use view::View;
use back;
use sessions::UserSession;
use oauth2::routes::authorize::{AuthorizeRequest, auth_redirect_url};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub name: String,
}

impl Permission {
    pub fn new(name: &str) -> Permission {
        Permission {
            name: name.to_owned(),
        }
    }
}

pub fn permissions_for_scopes(scopes: &[String]) -> Vec<Permission> {
    scopes.iter().map(|s| Permission::new(s)).collect()
}

/// called by user agent, redirected from authorize
/// should be logged in with cookie
/// if not logged in redirect to login
/// otherwise render consent page
/// on error set flash error and render consent page form
pub fn consent_get_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    let session = try!(UserSession::eval(req));
    let authenticated = session.map(|s| s.authenticated).unwrap_or(false); 
    let mut view = try!(View::new_for_session("consent.html", req));
     
    let params = try!(req.get::<UrlEncodedQuery>().map_err(OpenIdConnectError::from));
    
    let return_jwt = try!(multimap_get_one(&params, "return").map_err(OpenIdConnectError::from));
    let mut authorize_request = try!(AuthorizeRequest::decode(&return_jwt, &config.mac_signer));
           
                
    if !authenticated {
        let url = { try!(auth_redirect_url(req, "/login", &authorize_request)) };
    
        return Ok(Response::with((status::Found, Redirect(url))));
    }
                
    try!(authorize_request.load_client(&**config.application_repo));  
            
    view.data.insert("permissions".to_owned(), value::to_value(&permissions_for_scopes(&authorize_request.scopes)));
    view.data.insert("client".to_owned(), value::to_value(&authorize_request.client));    
    
    view.data.insert("return".to_owned(), value::to_value(&return_jwt));
            
    debug!("parsed query params: {:?}", params);
    
    Ok(Response::with((status::Ok, view.template())))
}

/// called by user agent form post
/// should be logged in with cookie
/// if not logged in, redirect to login form with flash error
/// otherwise redirect to caller
/// on error, set flash error and render consent form
pub fn consent_post_handler(req: &mut Request) -> IronResult<Response> {
    let config = try!(Config::get(req));
    let session = try!(UserSession::eval(req));
    let authenticated = session.map(|s| s.authenticated).unwrap_or(false);  
    let home_url = try!(relative_url(req, "/", None));
     
    let params = try!(req.get::<UrlEncodedBody>().map_err(OpenIdConnectError::from));
    let return_uri = try!(back::redirect_back(req, &params)).unwrap_or(home_url);
    
    let return_jwt = try!(multimap_get_one(&params, "return").map_err(OpenIdConnectError::from));
    let authorize_request = try!(AuthorizeRequest::decode(&return_jwt, &config.mac_signer));
                
    if !authenticated {
        let url = try!(auth_redirect_url(req, "/login", &authorize_request));
    
        return Ok(Response::with((status::Found, Redirect(url))));
    }
    
    // TODO save granted permissions

    Ok(Response::with((status::Found, Redirect(return_uri))))
}