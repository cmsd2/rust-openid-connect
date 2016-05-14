use std;
use std::fmt;

use serde;

use result::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GrantType {
    AuthorizationCode,
    ClientCredentials,
}

impl GrantType {
    pub fn from_str(s: &str) -> Result<GrantType> {
        match s {
            "authorization_code" => Ok(GrantType::AuthorizationCode),
            "client_credentials" => Ok(GrantType::ClientCredentials),
            _ => Err(OpenIdConnectError::UnknownGrantType(Box::new(s.to_owned())))
        }
    }
}

impl fmt::Display for GrantType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl serde::ser::Serialize for GrantType {
        fn serialize<S>(&self, serializer: &mut S) -> std::result::Result<(), S::Error>
        where S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl serde::de::Deserialize for GrantType {
        fn deserialize<D>(deserializer: &mut D) -> std::result::Result<GrantType, D::Error>
        where D: serde::de::Deserializer
    {
        deserializer.deserialize(GrantTypeVisitor)
    }
}

pub struct GrantTypeVisitor;

impl serde::de::Visitor for GrantTypeVisitor {
    type Value = GrantType;
    
    fn visit_str<E>(&mut self, s: &str) -> std::result::Result<GrantType, E> where E: serde::Error
    {
        GrantType::from_str(s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}