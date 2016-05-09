use std::collections::HashMap;

use serde::{Serializer, Deserializer};
use serde::de::Visitor;
use rbvt::result::ValidationError;
use rbvt::params::*;
use rbvt::state::*;
use chrono::*;

use result::Result as RoidcResult; // serde-codegen uses std Result which clashes
use authentication::*;
use serialisation::*;

#[derive(Clone, Debug)]//, Serialize, Deserialize)]
pub struct Grant {
    pub user_id: String,
    pub client_id: String,
    pub permissions_allowed: Vec<String>,
    pub permissions_denied: Vec<String>,
    
  //  #[serde(serialize_with="SerializeWith::serialize_with", deserialize_with="DeserializeWith::deserialize_with")]
    // #[serde(deserialize_with="DeserializeWith::deserialize_with")]
    // #[serde(serialize_with="SerializeWith::serialize_with")]
    pub created_at: DateTime<UTC>,
    
    //#[serde(serialize_with="SerializeWith::serialize_with", deserialize_with="DeserializeWith::deserialize_with")]
    // #[serde(deserialize_with="DeserializeWith::deserialize_with")]
    // #[serde(serialize_with="SerializeWith::serialize_with")]
    pub modified_at: DateTime<UTC>, // modified in oauth flow and by user managing perms
    
    // #[serde(serialize_with="SerializeWith::serialize_with", deserialize_with="DeserializeWith::deserialize_with")]
    // #[serde(deserialize_with="DeserializeWith::deserialize_with")]
    // #[serde(serialize_with="SerializeWith::serialize_with")]
    pub accessed_at: DateTime<UTC>, // updated when client accesses user data
}

impl Grant {
    pub fn new(user_id: String, client_id: String) -> Grant {
        let now = UTC::now();
        Grant {
            user_id: user_id,
            client_id: client_id,
            permissions_allowed: vec![],
            permissions_denied: vec![],
            created_at: now,
            modified_at: now,
            accessed_at: now,
        }
    }
    
    pub fn allowed_permissions(&self, requested_perms: &[String]) -> Vec<String> {
        let mut result = vec![];
        for p in requested_perms {
            if self.permissions_allowed.contains(p) {
                result.push(p.to_owned());
            }
        }
        result
    }
}
