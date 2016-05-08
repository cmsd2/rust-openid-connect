use iron::prelude::*;
use iron::status;

/// must be protected by tls
pub fn userinfo_get_handler(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "")))
}