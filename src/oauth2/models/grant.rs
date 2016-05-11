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

#[derive(Clone, Debug)]
pub struct GrantUpdate {
    pub user_id: String,
    pub client_id: String,
    pub permissions_added: Vec<String>,
    pub permissions_removed: Vec<String>,
}

impl GrantUpdate {
    pub fn new<U,C>(user_id: U, client_id: C) -> GrantUpdate where U: Into<String>, C: Into<String> {
        GrantUpdate {
            user_id: user_id.into(),
            client_id: client_id.into(),
            permissions_added: vec![],
            permissions_removed: vec![],
        }
    }
}

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
    
    pub fn new_for_update(update: GrantUpdate) -> Grant {
        let mut g = Grant::new(update.user_id, update.client_id);
        g.permissions_allowed = update.permissions_added;
        g.permissions_denied = update.permissions_removed;
        g
    }
    
    pub fn update(&mut self, update: GrantUpdate) {
        let now = UTC::now();
        self.modified_at = now;
        self.accessed_at = now;
        Self::merge(&mut self.permissions_allowed, update.permissions_added);
        Self::merge(&mut self.permissions_denied, update.permissions_removed);
    }
    
    pub fn merge(a: &mut Vec<String>, b: Vec<String>) {
        //TODO fix array merge
        for s in b {
            if !a.contains(&s) {
                a.push(s)
            }
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
