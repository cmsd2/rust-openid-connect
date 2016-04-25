use iron::prelude::*;
use router::Router;

use validation::params;
use result::Result;

pub fn get_url_param(req: &mut Request, name: &str) -> Result<String> {
    let params = try!(req.extensions.get::<Router>().ok_or(params::ParamError::NotFound("id".to_owned())));
    
    let value = try!(params.find(name).map(|s| s.to_owned()).ok_or(params::ParamError::NotFound("id".to_owned())));
    
    Ok(value)
}