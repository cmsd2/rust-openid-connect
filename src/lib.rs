// for large numbers of errors definitions with quick-error
#![recursion_limit="200"]

extern crate iron;
extern crate iron_sessionstorage;
extern crate router;
extern crate bodyparser;
extern crate urlencoded;
extern crate handlebars_iron;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate rustc_serialize;
extern crate rand;
extern crate url;
extern crate cookie;
extern crate persistent;
extern crate plugin;
extern crate rbvt;
extern crate jsonwebtoken;
extern crate chrono;
extern crate cast;
extern crate openssl;
extern crate crypto;

pub mod result;
pub mod urls;
pub mod routes;
pub mod authentication;
pub mod users;
pub mod config;
pub mod handlers;
pub mod login_manager;
pub mod sessions;
pub mod view;
pub mod helpers;
pub mod oauth2;
pub mod service;
pub mod response_type;
pub mod back;
pub mod serialisation;
pub mod response_mode;
pub mod site_config;
pub mod x_headers;
pub mod grant_type;
pub mod truthy;


#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
