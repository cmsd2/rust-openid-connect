extern crate iron;
extern crate urlencoded;
#[macro_use] extern crate quick_error;

pub mod result;
pub mod params;

#[derive(Copy, Clone, Debug)]
pub enum ResponseType {
    Code,
}

#[derive(Clone, Debug)]
pub struct AuthorizeRequest {
    response_type: ResponseType,
    scopes: Vec<String>,
    client_id: String,
    state: String,
    // nonce: String, // ?
    redirect_uri: String, // or url type?
}
    
#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
