use iron::prelude::*;
use iron::status;

use config::Config;

/// /.well-known/openid-configuration
pub fn openid_config_get_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "")))
}