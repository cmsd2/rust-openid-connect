use iron::prelude::*;
use iron::status;

use config::Config;

/// must be protected by tls
pub fn userinfo_get_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "")))
}