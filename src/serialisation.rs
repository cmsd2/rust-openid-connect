use std::marker::PhantomData;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{Error, Visitor};
use chrono::*;
use cast;
use std;

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
