use iron::prelude::*;
use iron::status;
use view::View;
use serde_json::value;
use result::OpenIdConnectError;

pub fn home_handler(req: &mut Request) -> IronResult<Response> {
    let mut resp = Response::new();

    let mut view = try!(View::new_for_session("index.html", req));
    
    view.data.insert("msg".to_owned(), try!(value::to_value("Hello, World!").map_err(OpenIdConnectError::from)));
    
    resp.set_mut(try!(view.template().map_err(OpenIdConnectError::from))).set_mut(status::Ok);
    
    Ok(resp)
}