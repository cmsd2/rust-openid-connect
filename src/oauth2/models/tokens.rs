use std::fmt::{self, Display, Formatter};

use serde;
use serde::de::Deserialize;
use serde_json;
use chrono::*;
use site_config::*;
use result::OpenIdConnectError;

#[derive(Copy, Clone, Debug)]
pub enum TokenType {
    Bearer,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            TokenType::Bearer => write!(f, "Bearer"),
        }
    }
}

impl serde::Serialize for TokenType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer,
    {
        let s = format!("{}", self);
        
        serializer.serialize_str(&s)
    }
}

impl Deserialize for TokenType {
    fn deserialize<D>(deserializer: D) -> Result<TokenType, D::Error>
        where D: serde::Deserializer,
    {
        deserializer.deserialize(TokenTypeVisitor)
    }
}

struct TokenTypeVisitor;

impl serde::de::Visitor for TokenTypeVisitor {
    type Value = TokenType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("token type")
    }

    fn visit_string<E>(self, value: String) -> Result<TokenType, E>
        where E: serde::de::Error,
    {
        match &value[..] {
            "Bearer" => Ok(TokenType::Bearer),
            _ => Err(serde::de::Error::custom("unexpected token_type"))
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Token {
    pub code: Option<String>,
    pub access_token: Option<String>,
    pub token_type: Option<TokenType>,
    pub refresh_token: Option<String>,
    pub expires_in: TokenDuration,
    pub id_token: Option<String>,
    pub state: Option<String>,
}

impl Token {
    pub fn new(code: Option<String>, token_type: Option<TokenType>, access_token: Option<String>, refresh_token: Option<String>, expires_in: Duration, id_token: Option<String>, state: Option<String>) -> Token {
        Token {
            code: code,
            access_token: access_token,
            token_type: token_type,
            refresh_token: refresh_token,
            expires_in: expires_in.into(),
            id_token: id_token,
            state: state,
        }
    }
    
    pub fn query_pairs(&self) -> Result<Vec<(String, String)>, OpenIdConnectError> {
        let mut qp = vec![];
        let json = try!(serde_json::to_value(self));
        for (k,v) in json.as_object().unwrap() {
            if !v.is_null() {
                if v.is_number() {
                    qp.push((k.to_owned(), format!("{}", v.as_i64().unwrap())));
                } else if v.is_boolean() {
                    if v.as_bool().unwrap() {
                        qp.push((k.to_owned(), "true".to_owned()));
                    } else {
                        qp.push((k.to_owned(), "false".to_owned()));
                    }
                } else if v.is_string() {
                    qp.push((k.to_owned(), v.as_str().map(|s| s.to_owned()).unwrap()));
                } else {
                    debug!("can't serialize thing {}={:?}", k, v);
                    unimplemented!();
                }
            }
        }
        Ok(qp)
    }
}

impl serde::ser::Serialize for Token {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = try!(serializer.serialize_map(None));

        if let Some(ref code) = self.code {
            try!(map.serialize_entry("code", code));
        }
        
        if let Some(ref access_token) = self.access_token {
            try!(map.serialize_entry("access_token", access_token));
        }
        
        try!(map.serialize_entry("token_type", &self.token_type));
        
        if let Some(ref refresh_token) = self.refresh_token {
            try!(map.serialize_entry("refresh_token", refresh_token));
        }
        
        if let Some(ref id_token) = self.id_token {
            try!(map.serialize_entry("id_token", id_token));
        }
        
        if let Some(ref state) = self.state {
            try!(map.serialize_entry("state", state));
        }
        
        try!(map.serialize_entry("expires_in", &self.expires_in));
        
        map.end()
    }
}