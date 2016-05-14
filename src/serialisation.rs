use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde;
use chrono::*;
use std;
use iron::Url;

use result::OpenIdConnectError;

pub trait SerializeWith: Sized {
    fn serialize_with<S>(&self, ser: &mut S) -> std::result::Result<(), S::Error>
        where S: Serializer;
}

pub trait DeserializeWith: Sized {
    fn deserialize_with<D>(de: &mut D) -> std::result::Result<Self, D::Error>
        where D: Deserializer;
}

impl SerializeWith for DateTime<UTC> {
    fn serialize_with<S>(&self, serializer: &mut S) -> std::result::Result<(), S::Error> where S: Serializer {
        self.timestamp().serialize(serializer)
    }
}
 
impl DeserializeWith for DateTime<UTC> {
    fn deserialize_with<D>(deserializer: &mut D) -> std::result::Result<DateTime<UTC>, D::Error> where D: Deserializer {
        Ok(UTC.timestamp(try!(i64::deserialize(deserializer)), 0))
    }
}

impl SerializeWith for Url {
    fn serialize_with<S>(&self, serializer: &mut S) -> std::result::Result<(), S::Error> where S: Serializer {
        self.to_string().serialize(serializer)
    }
}
 
impl DeserializeWith for Url {
    fn deserialize_with<D>(deserializer: &mut D) -> std::result::Result<Url, D::Error> where D: Deserializer {
        let url_str = try!(String::deserialize(deserializer));
        
        let url = try!(Url::parse(&url_str[..]).map_err(|e| OpenIdConnectError::UrlError(e)).map_err(|e| serde::de::Error::custom(e.to_string())));
        
        Ok(url)
    }
}

impl DeserializeWith for Option<Url> {
    fn deserialize_with<D>(deserializer: &mut D) -> std::result::Result<Option<Url>, D::Error> where D: Deserializer {
        let maybe_url_str: Option<String> = try!(Option::deserialize(deserializer));
        
        if let Some(url_str) = maybe_url_str {
            let url = try!(Url::parse(&url_str[..]).map_err(|e| OpenIdConnectError::UrlError(e)).map_err(|e| serde::de::Error::custom(e.to_string())));
        
            Ok(Some(url))
        } else {
            Ok(None)
        }
    }
}

impl <T> SerializeWith for Option<T> where T: SerializeWith {
    fn serialize_with<S>(&self, serializer: &mut S) -> std::result::Result<(), S::Error> where S: Serializer {
        if let Some(obj) = self.as_ref() {
            obj.serialize_with(serializer)
        } else {
            serializer.serialize_unit()
        }
    }
}
/*
struct OptionDeVisitor<T> where T: DeserializeWith {
    phantom_data: PhantomData<T>
}

impl <T> OptionDeVisitor<T> where T: DeserializeWith {
    pub fn new() -> OptionDeVisitor<T> {
        OptionDeVisitor {
            phantom_data: PhantomData::new(),
        }
    }
}

impl <T> serde::de::Visitor for OptionDeVisitor<T> where T: DeserializeWith {
    type Value = Option<T>;
    
    fn visit_none<E>(&mut self) -> Result<Self::Value, E> where E: serde::de::Error {
        None
    }
    
    fn visit_some<D>(&mut self, deserializer: &mut D) -> Result<Self::Value, D::Error> where D: Deserializer {
        Some(try!(T::deserialize_with(deserializer)))
    }
}
 
impl <T> DeserializeWith for Option<T> where T: DeserializeWith {
    fn deserialize_with<D>(deserializer: &mut D) -> std::result::Result<Option<T>, D::Error> where D: Deserializer {
        deserializer.deserialize_option(OptionDeVisitor::new())
    }
}
*/