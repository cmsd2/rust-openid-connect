use iron::prelude::*;
use iron::status;

/// must be protected by tls
pub fn userinfo_get_handler(_req: &mut Request) -> IronResult<Response> {
    //TODO return some user info
    Ok(Response::with((status::Ok, "")))
}