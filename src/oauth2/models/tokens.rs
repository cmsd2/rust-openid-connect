use std::fmt::{self, Display, Formatter};
use serde;
use serde::de::Deserialize;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub token_type: TokenType,
    pub refresh_token: Option<String>,
    pub expires_in: u32,
    pub id_token: Option<String>,
}