use iron::prelude::*;
use iron::status;
use config::Config;
use view::View;
use serde_json::value;

pub fn home_handler(_config: &Config, req: &mut Request) -> IronResult<Response> {
    let mut resp = Response::new();

    let mut view = try!(View::new_for_session("index.html", req));
    
    view.data.insert("msg".to_owned(), value::to_value("Hello, World!"));
    
    resp.set_mut(view.template()).set_mut(status::Ok);
    
    Ok(resp)
}