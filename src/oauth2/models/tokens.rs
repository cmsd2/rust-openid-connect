use std::fmt::{self, Display, Formatter};

use serde;
use serde::de::Deserialize;
use serde_json;
use chrono::*;
use site_config::*;

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
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer,
    {
        let s = format!("{}", self);
        
        serializer.serialize_str(&s)
    }
}

impl Deserialize for TokenType {
    fn deserialize<D>(deserializer: &mut D) -> Result<TokenType, D::Error>
        where D: serde::Deserializer,
    {
        deserializer.deserialize(TokenTypeVisitor)
    }
}

struct TokenTypeVisitor;

impl serde::de::Visitor for TokenTypeVisitor {
    type Value = TokenType;

    fn visit_string<E>(&mut self, value: String) -> Result<TokenType, E>
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
    pub token_type: TokenType,
    pub refresh_token: Option<String>,
    pub expires_in: TokenDuration,
    pub id_token: Option<String>,
    pub state: Option<String>,
}

impl Token {
    pub fn new(code: Option<String>, access_token: Option<String>, refresh_token: Option<String>, expires_in: Duration, id_token: Option<String>, state: Option<String>) -> Token {
        Token {
            code: code,
            access_token: access_token,
            token_type: TokenType::Bearer,
            refresh_token: refresh_token,
            expires_in: expires_in.into(),
            id_token: id_token,
            state: state,
        }
    }
    
    pub fn query_pairs(&self) -> Vec<(String, String)> {
        let mut qp = vec![];
        let json = serde_json::to_value(self);
        for (k,v) in json.as_object().unwrap() {
            if !v.is_null() {
                if v.is_number() {
                    qp.push((k.to_owned(), format!("{}", v.as_i64().unwrap())));
                } else if v.is_boolean() {
                    if v.as_boolean().unwrap() {
                        qp.push((k.to_owned(), "true".to_owned()));
                    } else {
                        qp.push((k.to_owned(), "false".to_owned()));
                    }
                } else if v.is_string() {
                    qp.push((k.to_owned(), v.as_string().map(|s| s.to_owned()).unwrap()));
                } else {
                    debug!("can't serialize thing {}={:?}", k, v);
                    unimplemented!();
                }
            }
        }
        qp
    }
}

impl serde::ser::Serialize for Token {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer,
    {
        serializer.serialize_map(TokenSerVisitor(&self))
    }
}

struct TokenSerVisitor<'a>(&'a Token);

impl<'a> serde::ser::MapVisitor for TokenSerVisitor<'a> {
    fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
        where S: serde::Serializer
    {        
        if let Some(ref code) = self.0.code {
            try!(serializer.serialize_map_elt("code", code));
        }
        
        if let Some(ref access_token) = self.0.access_token {
            try!(serializer.serialize_map_elt("access_token", access_token));
        }
        
        try!(serializer.serialize_map_elt("token_type", self.0.token_type));
        
        if let Some(ref refresh_token) = self.0.refresh_token {
            try!(serializer.serialize_map_elt("refresh_token", refresh_token));
        }
        
        if let Some(ref id_token) = self.0.id_token {
            try!(serializer.serialize_map_elt("id_token", id_token));
        }
        
        if let Some(ref state) = self.0.code {
            try!(serializer.serialize_map_elt("state", state));
        }
        
        try!(serializer.serialize_map_elt("expires_in", self.0.expires_in));
        
        Ok(None)
    }
}