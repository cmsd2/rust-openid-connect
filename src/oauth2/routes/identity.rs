use iron::prelude::*;
use iron::status;

use config::Config;

/// used by identityserver3 samples. similar to userinfo
/// must be protected by tls
pub fn identity_get_handler(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "")))
}