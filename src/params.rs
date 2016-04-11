use std::result;
use std::collections::HashMap;

quick_error! {
    #[derive(Debug)]
    pub enum ParamError {
        NotFound(name: Box<String>) {
            description("param not found")
            display("Param not found: {}", name)
        }
        
        MultipleValues(name: Box<String>) {
            description("multiple values found")
            display("Multiple values found for param: {}", name)
        }
        
        BadValue(name: Box<String>) {
            description("param has bad value")
            display("Bad value found for param: {}", name)
        }
    }
}

pub type ParamsResult<T> = result::Result<T,ParamError>;

pub fn multimap_get<'a>(mm: &'a HashMap<String, Vec<String>>, key: &str) -> ParamsResult<&'a Vec<String>> {
    match mm.get(key) {
        Some(values) => {
            if values.is_empty() {
                Err(ParamError::NotFound(Box::new(key.to_owned())))
            } else {
                Ok(values)
            }
        },
        None => Err(ParamError::NotFound(Box::new(key.to_owned())))
    }
}

pub fn multimap_get_one<'a>(mm: &'a HashMap<String, Vec<String>>, key: &str) -> ParamsResult<&'a str> {
    let values = try!(multimap_get(mm, key));
    
    if values.len() == 1 {
        Ok(values.first().map(|s| &s[..]).unwrap())
    } else {
        Err(ParamError::MultipleValues(Box::new(key.to_owned())))
    }
}