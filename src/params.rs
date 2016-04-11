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
    multimap_get_maybe(mm, key).ok_or(ParamError::NotFound(Box::new(key.to_owned())))
}

pub fn multimap_get_one<'a>(mm: &'a HashMap<String, Vec<String>>, key: &str) -> ParamsResult<&'a str> {
    let maybe_one = try!(multimap_get_maybe_one(mm, key));
    
    maybe_one.ok_or(ParamError::NotFound(Box::new(key.to_owned())))
}

pub fn multimap_get_maybe<'a>(mm: &'a HashMap<String, Vec<String>>, key: &str) -> Option<&'a Vec<String>> {
    match mm.get(key) {
        Some(values) => {
            if values.is_empty() {
                None
            } else {
                Some(values)
            }
        },
        None => None
    }
}

pub fn multimap_get_maybe_one<'a>(mm: &'a HashMap<String, Vec<String>>, key: &str) -> ParamsResult<Option<&'a str>> {
    match multimap_get_maybe(mm, key) {
        Some(values) => {
            if values.len() == 1 {
                Ok(Some(values.first().map(|s| &s[..]).unwrap()))
            } else {
                Err(ParamError::MultipleValues(Box::new(key.to_owned())))
            }
        },
        None => Ok(None)
    }
}