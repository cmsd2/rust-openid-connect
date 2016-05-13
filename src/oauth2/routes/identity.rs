use iron::prelude::*;
use iron::status;

/// used by identityserver3 samples. similar to userinfo
/// must be protected by tls
pub fn identity_get_handler(_req: &mut Request) -> IronResult<Response> {
    //TODO return some user info
    Ok(Response::with((status::Ok, "")))
}