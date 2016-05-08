use result::{Result, OpenIdConnectError};
use serde;
use std;
use std::fmt;

/// Authorization Code flow: "code"
/// Implicit flow: "id_token" or "id_token token"
/// Hybrid flow: "code id_token" or "code token" or "code id_token token"
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ResponseType {
    pub code: bool,
    pub id_token: bool,
    pub token: bool,
}

impl ResponseType {
    pub fn new(code: bool, id_token: bool, token: bool) -> ResponseType {
        ResponseType {
            code: code,
            id_token: id_token,
            token: token,
        }
    }
    
    pub fn from_str(s: &str) -> Result<ResponseType> {
        let parts = s.split(" ");
        
        let mut r = ResponseType::new(false, false, false);
        
        for part in parts {
            match part {
                "code" => { r.code = true; }
                "id_token" => { r.id_token = true; }
                "token" => { r.token = true; }
                "none" => { r.code = false; r.id_token = false; r.token = false; }
                _other => { return Err(OpenIdConnectError::UnknownResponseType(Box::new(s.to_owned()))); }
            }
        }
        
        Ok(r)
    }
}

impl fmt::Display for ResponseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut types = vec![];
        if self.code {
            types.push("code");
        }
        
        if self.token {
            types.push("token");
        }
        
        if self.id_token {
            types.push("id_token");
        }
        
        let s = if types.is_empty() {
            "none".to_owned()
        } else {
            types.join(" ")
        };
        
        write!(f, "{}", s)
    }
}

impl serde::ser::Serialize for ResponseType {
        fn serialize<S>(&self, serializer: &mut S) -> std::result::Result<(), S::Error>
        where S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl serde::de::Deserialize for ResponseType {
        fn deserialize<D>(deserializer: &mut D) -> std::result::Result<ResponseType, D::Error>
        where D: serde::de::Deserializer
    {
        deserializer.deserialize(ResponseTypeVisitor)
    }
}

pub struct ResponseTypeVisitor;

impl serde::de::Visitor for ResponseTypeVisitor {
    type Value = ResponseType;
    
    fn visit_str<E>(&mut self, s: &str) -> std::result::Result<ResponseType, E> where E: serde::Error
    {
        ResponseType::from_str(s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

#[cfg(test)]
mod test {
    use serde_json;
    use super::*;
    
    #[test]
    fn test_response_type_ser() {
        let r = ResponseType::new(false, false, false);
        assert_eq!(r.to_string(), "none");
        assert_eq!(serde_json::to_string(&r).unwrap(), r#""none""#);
        
        let r = ResponseType::new(true, true, true);
        assert_eq!(r.to_string(), "code token id_token");
        assert_eq!(serde_json::to_string(&r).unwrap(), r#""code token id_token""#);
    }
    
    #[test]
    fn test_reponse_type_de() {
        let r = ResponseType::new(false, false, false);
        assert_eq!(ResponseType::from_str("none").unwrap(), r);
        assert_eq!(serde_json::from_str::<ResponseType>(r#""none""#).unwrap(), r);
        
        let r = ResponseType::new(true, true, true);
        assert_eq!(ResponseType::from_str("code token id_token").unwrap(), r);
        assert_eq!(serde_json::from_str::<ResponseType>(r#""code token id_token""#).unwrap(), r);
    }
}