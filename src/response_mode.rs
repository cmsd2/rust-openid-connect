use std;
use std::fmt;

use serde;

use result::*;
use response_type::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ResponseMode {
    Query,
    Fragment,
}

impl ResponseMode {
    pub fn from_str(s: &str) -> Result<ResponseMode> {
        match s {
            "query" => Ok(ResponseMode::Query),
            "fragment" => Ok(ResponseMode::Fragment),
            s => Err(OpenIdConnectError::UnknownResponseMode(Box::new(s.to_owned())))
        }
    }
    
    pub fn default_for_response_type(rt: ResponseType) -> ResponseMode {
        /*
        code                   query
        token                  fragment
        id_token               fragment (must not use query)
        none                   query
        code token             fragment (must not use query)
        code id_token          fragment (must not use query)
        id_token token         fragment (must not use query)
        code id_token token    fragment (must not use query)
        */
        if !rt.token && !rt.id_token {
            ResponseMode::Query
        } else {
            ResponseMode::Fragment
        }
    }
    
    pub fn validate_response_mode(rm: ResponseMode, rt: ResponseType) -> Result<()> {
        if rm == ResponseMode::Query && (rt.token || rt.id_token) {
            Err(OpenIdConnectError::ResponseModeUnavailable)
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for ResponseMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ResponseMode::Query => write!(f, "query"),
            ResponseMode::Fragment => write!(f, "fragment"),
        }
    }
}

impl serde::ser::Serialize for ResponseMode {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl serde::de::Deserialize for ResponseMode {
        fn deserialize<D>(deserializer: D) -> std::result::Result<ResponseMode, D::Error>
        where D: serde::de::Deserializer
    {
        deserializer.deserialize(ResponseModeVisitor)
    }
}

pub struct ResponseModeVisitor;

impl serde::de::Visitor for ResponseModeVisitor {
    type Value = ResponseMode;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("response_mode")
    }
    
    fn visit_str<E>(self, s: &str) -> std::result::Result<ResponseMode, E> where E: serde::de::Error
    {
        ResponseMode::from_str(s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}