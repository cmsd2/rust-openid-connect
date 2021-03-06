use std;
use std::fmt;

use serde::{Serializer, Deserializer};
use serde;
use jsonwebtoken::validation::*;
use chrono::*;
use result::OpenIdConnectError;

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

#[derive(Clone, Debug)]
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

struct GrantSerVisitor<'a>(&'a Grant);

impl serde::Serialize for Grant {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut struc = try!(serializer.serialize_struct("grant", 7));

        try!(struc.serialize_field("user_id", &self.user_id));
        try!(struc.serialize_field("client_id", &self.client_id));
        try!(struc.serialize_field("permissions_allowed", &self.permissions_allowed));
        try!(struc.serialize_field("permissions_denied", &self.permissions_denied));
        try!(struc.serialize_field("created_at", &self.created_at.timestamp()));
        try!(struc.serialize_field("modified_at", &self.modified_at.timestamp()));
        try!(struc.serialize_field("accessed_at", &self.accessed_at.timestamp()));

        struc.end()
    }
}

impl serde::Deserialize for Grant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer {
        deserializer.deserialize(GrantDeVisitor)
    }
}

struct GrantDeVisitor;

impl serde::de::Visitor for GrantDeVisitor {
    type Value = Grant;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("grant")
    }
    
    fn visit_map<V>(self, mut visitor: V) -> Result<Grant, V::Error>
        where V: serde::de::MapVisitor,
    {
        let mut user_id: Option<String> = None;
        let mut client_id: Option<String> = None;
        let mut permissions_allowed: Option<Vec<String>> = None;
        let mut permissions_denied: Option<Vec<String>> = None;
        let mut created_at: Option<i64> = None;
        let mut modified_at: Option<i64> = None;
        let mut accessed_at: Option<i64> = None;
        
        loop {
            if let Some(key) = try!(visitor.visit_key::<String>()) {
                match &key[..] {
                    "user_id" => { user_id = try!(visitor.visit_value()); },
                    "client_id" => { client_id = try!(visitor.visit_value()); },
                    "permissions_allowed" => { permissions_allowed = try!(visitor.visit_value()); },
                    "permissions_denied" => { permissions_denied = try!(visitor.visit_value()); },
                    "created_at" => { created_at = try!(visitor.visit_value()); },
                    "modified_at" => { modified_at = try!(visitor.visit_value()); },
                    "accessed_at" => { accessed_at = try!(visitor.visit_value()); },
                    _ => {},
                }
            } else {
                break;
            }
        }
        
        let mut vs = ValidationState::new();
        
        if user_id.is_none() {
            vs.reject("user_id", ValidationError::MissingRequiredValue("user_id".to_owned()));
        }
        
        if client_id.is_none() {
            vs.reject("client_id", ValidationError::MissingRequiredValue("client_id".to_owned()));
        }
        
        if vs.valid {
            let mut g = Grant::new(user_id.unwrap(), client_id.unwrap());
            
            if let Some(p) = permissions_allowed {
                g.permissions_allowed = p;
            }
            
            if let Some(p) = permissions_denied {
                g.permissions_denied = p;
            }
            
            if let Some(t) = created_at {
                g.created_at = UTC.timestamp(t, 0);
            }
            
            if let Some(t) = modified_at {
                g.modified_at = UTC.timestamp(t, 0);
            }
            
            if let Some(t) = accessed_at {
                g.accessed_at = UTC.timestamp(t, 0);
            }
            
            Ok(g)
        } else {
            Err(serde::de::Error::custom(format!("{}", OpenIdConnectError::from(ValidationError::ValidationError(vs)))))
        }
    }
}