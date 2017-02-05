use std::ops::Deref;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde;
use chrono::*;
use std;
use iron::Url;

use result::OpenIdConnectError;

#[derive(Clone, Debug)]
pub struct UTCDateTime {
    date_time: DateTime<UTC>,
}

impl UTCDateTime {
    pub fn new(dt: DateTime<UTC>) -> UTCDateTime {
        UTCDateTime {
            date_time: dt
        }
    }
}

impl From<DateTime<UTC>> for UTCDateTime {
    fn from(dt: DateTime<UTC>) -> UTCDateTime {
        UTCDateTime::new(dt)
    }
}

impl Into<DateTime<UTC>> for UTCDateTime {
    fn into(self) -> DateTime<UTC> {
        self.date_time
    }
}

impl Deref for UTCDateTime {
    type Target = DateTime<UTC>;
    
    fn deref(&self) -> &DateTime<UTC> {
        &self.date_time
    }
}

impl serde::ser::Serialize for UTCDateTime {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where S: serde::Serializer,
    {
        self.date_time.serialize_with(serializer)
    }
}

impl serde::de::Deserialize for UTCDateTime {
    fn deserialize<D>(deserializer: D) -> std::result::Result<UTCDateTime, D::Error>
        where D: serde::de::Deserializer
    {
        let date_time: DateTime<UTC> = try!(DeserializeWith::deserialize_with(deserializer));
        
        Ok(UTCDateTime::new(date_time))
    }
}

pub trait SerializeWith: Sized {
    fn serialize_with<S>(&self, ser: S) -> std::result::Result<S::Ok, S::Error>
        where S: Serializer;
}

pub trait DeserializeWith: Sized {
    fn deserialize_with<D>(de: D) -> std::result::Result<Self, D::Error>
        where D: Deserializer;
}

impl SerializeWith for DateTime<UTC> {
    fn serialize_with<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
        self.timestamp().serialize(serializer)
    }
}
 
impl DeserializeWith for DateTime<UTC> {
    fn deserialize_with<D>(deserializer: D) -> std::result::Result<DateTime<UTC>, D::Error> where D: Deserializer {
        Ok(UTC.timestamp(try!(i64::deserialize(deserializer)), 0))
    }
}

impl SerializeWith for Duration {
    fn serialize_with<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
        self.num_seconds().serialize(serializer)
    }
}
 
impl DeserializeWith for Duration {
    fn deserialize_with<D>(deserializer: D) -> std::result::Result<Duration, D::Error> where D: Deserializer {
        let d = try!(i64::deserialize(deserializer));
        
        Ok(Duration::seconds(d))
    }
}

impl SerializeWith for Url {
    fn serialize_with<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
        self.to_string().serialize(serializer)
    }
}
 
impl DeserializeWith for Url {
    fn deserialize_with<D>(deserializer: D) -> std::result::Result<Url, D::Error> where D: Deserializer {
        let url_str = try!(String::deserialize(deserializer));
        
        let url = try!(Url::parse(&url_str[..]).map_err(|e| OpenIdConnectError::UrlError(e)).map_err(|e| serde::de::Error::custom(e.to_string())));
        
        Ok(url)
    }
}

impl DeserializeWith for Option<Url> {
    fn deserialize_with<D>(deserializer: D) -> std::result::Result<Option<Url>, D::Error> where D: Deserializer {
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
    fn serialize_with<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
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