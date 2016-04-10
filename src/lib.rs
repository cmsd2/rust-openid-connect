#[macro_use] extern crate quick_error;

pub mod result;

pub enum ResponseType {
    Code,
}
    
#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
