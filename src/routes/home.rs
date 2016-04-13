use iron::prelude::*;
use iron::status;
use handlebars_iron::Template;
use std::collections::HashMap;
use config::Config;

pub fn home_handler(config: &Config, _: &mut Request) -> IronResult<Response> {
    let mut resp = Response::new();

    let mut data = HashMap::<String,String>::new();
    data.insert("msg".to_owned(), "Hello, World!".to_owned());
    data.insert("_view".to_owned(), "index.html".to_owned());
    resp.set_mut(Template::new("_layout.html", data)).set_mut(status::Ok);
    Ok(resp)
}