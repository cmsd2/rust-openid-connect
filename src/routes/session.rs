use std::io::Read;

use serde_json;
use iron::prelude::*;
use iron::status;

use config::Config;
use result::*;
use login_manager::*;
use sessions::*;

pub fn parse_credentials(req: &mut Request) -> Result<Credentials> {
    let mut creds_str = String::new();
    
    try!(req.body.read_to_string(&mut creds_str));
    
    let creds = try!(serde_json::from_str(&creds_str));
    
    Ok(creds)
}

pub fn serialize_session(session: &UserSession) -> Result<String> {
    let json_str = try!(serde_json::to_string(&session));
    
    Ok(json_str)
}

pub fn session_get_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
    let login = try!(config.session_controller.load_session(req));
    
    if login.session.is_some() {
        let session_json = try!(serialize_session(&login.session.unwrap()));
    
        Ok(Response::new()
            .set(status::Ok)
            .set(session_json)) 
    } else {
        Ok(Response::new()
            .set(status::NotFound)
            .set("Not Found"))
    }
}

pub fn session_post_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
    let creds = try!(parse_credentials(req));
    
    debug!("received credentials: {:?}", creds);
    
    match config.session_controller.sessions.authenticate(&creds) {
        Ok(session) => {
            let session_json = try!(serialize_session(&session));
            
            Ok(Response::new()
                .set(status::Ok)
                .set(Login::new(&config.session_controller.login_manager.config, Some(session)).cookie())
                .set(session_json))
        },
        Err(OpenIdConnectError::InvalidUsernameOrPassword) => {
            Ok(Response::with((status::Forbidden, "invalid username or password")))   
        },    
        _ => {
            Ok(Response::with((status::InternalServerError, "post session not implemented")))
        }
    }      
}

pub fn session_delete_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
    try!(config.session_controller.clear_session(req));
    
    Ok(Response::with((status::Ok, "not implemented")))
}