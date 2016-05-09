use std::collections::HashMap;

use serde::{Serializer, Deserializer};
use rbvt::result::ValidationError;
use rbvt::params::*;
use rbvt::state::*;

use result::Result;
use authentication::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientApplication {
    pub name: Option<String>,
    pub client_id: String,
    secret: Option<String>,
    pub hashed_secret: Option<String>,
    pub redirect_uris: Vec<String>,
}

impl ClientApplication {
    pub fn new(client_id: String, secret: Option<String>) -> ClientApplication {
        ClientApplication {
            name: None,
            client_id: client_id,
            hashed_secret: Some(hash_password(secret.as_ref().map(|s| &s[..]).unwrap_or(""))),
            secret: secret,
            redirect_uris: vec![],
        }
    }
    
    pub fn match_redirect_uri(&self, redirect_uri: &str) -> bool {
        self.redirect_uris.iter().find(|s| &s[..] == redirect_uri).is_some()
    }
}

#[derive(Clone, Debug)]
pub struct ClientApplicationBuilder {
    pub name: Option<String>,
    pub client_id: Option<String>,
    pub secret: Option<String>,
    pub hashed_secret: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    
    pub validation_state: ValidationState,
}

impl ClientApplicationBuilder {
    pub fn new() -> ClientApplicationBuilder {
        ClientApplicationBuilder {
            name: None,
            client_id: None,
            secret: None,
            hashed_secret: None,
            redirect_uris: None,
            validation_state: ValidationState::new(),
        }
    }
    
    pub fn build(self) -> Result<ClientApplication> {
        Ok(ClientApplication {
            name: self.name,
            client_id: try!(self.client_id.ok_or(ValidationError::MissingRequiredValue("client_id".to_owned()))),
            secret: self.secret,
            hashed_secret: self.hashed_secret,
            redirect_uris: self.redirect_uris.unwrap_or(vec![]),
        })
    }
    
    pub fn load_params(&mut self, params: &HashMap<String, Vec<String>>) -> Result<()> {
        self.name = try!(multimap_get_maybe_one(params, "name")).map(|s| s.to_owned());
        
        if let Some(client_id) = try!(multimap_get_maybe_one(params, "client_id")) {
            self.client_id = Some(client_id.to_owned()); 
        }
        
        self.secret = try!(multimap_get_maybe_one(params, "secret")).map(|s| s.to_owned());
        self.hashed_secret = try!(multimap_get_maybe_one(params, "hashed_secret")).map(|s| s.to_owned());
        
        self.redirect_uris = params.get("redirect_uris").map(|r| r.to_owned().into_iter().filter(|r| !r.is_empty()).collect());
        
        Ok(())
    }
    
    pub fn validate(&mut self) -> Result<bool> {
        self.validation_state = ValidationState::new();
        
        if !self.client_id.is_some() {
            self.validation_state.reject("client_id", ValidationError::MissingRequiredValue("client_id".to_owned()));
        }
        
        Ok(self.validation_state.valid)
    }
    
    pub fn build_from_params(params: &HashMap<String, Vec<String>>) -> Result<ClientApplication> {
        let mut builder = ClientApplicationBuilder::new();
        
        try!(builder.load_params(params));
        
        try!(builder.validate());
        
        builder.build()
    }
}
