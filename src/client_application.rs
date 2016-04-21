use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use vlad::result::VladError;
use vlad::params::*;
use vlad::state::*;

use result::{Result, OpenIdConnectError};
use authentication::*;

#[derive(Clone,Debug)]
pub struct ClientApplication {
    pub client_id: String,
    pub secret: Option<String>,
    pub hashed_secret: Option<String>,
    pub redirect_uris: Vec<String>,
}

impl ClientApplication {
    pub fn new(client_id: String, secret: Option<String>) -> ClientApplication {
        ClientApplication {
            client_id: client_id,
            hashed_secret: Some(hash_password(secret.as_ref().map(|s| &s[..]).unwrap_or(""))),
            secret: secret,
            redirect_uris: vec![],
        }
    }
}

#[derive(Clone, Debug)]
pub struct ClientApplicationBuilder {
    client_id: Option<String>,
    secret: Option<String>,
    hashed_secret: Option<String>,
    redirect_uris: Vec<String>,
    
    validation_state: ValidationState,
}

impl ClientApplicationBuilder {
    pub fn new() -> ClientApplicationBuilder {
        ClientApplicationBuilder {
            client_id: None,
            secret: None,
            hashed_secret: None,
            redirect_uris: vec![],
            validation_state: ValidationState::new(),
        }
    }
    
    pub fn build(self) -> Result<ClientApplication> {
        Ok(ClientApplication {
            client_id: try!(self.client_id.ok_or(VladError::MissingRequiredValue("client_id".to_owned()))),
            secret: self.secret,
            hashed_secret: self.hashed_secret,
            redirect_uris: self.redirect_uris,
        })
    }
    
    pub fn load_params(&mut self, params: &HashMap<String, Vec<String>>) -> Result<bool> {
        if let Some(client_id) = try!(multimap_get_maybe_one(params, "client_id")) {
            self.client_id = Some(client_id.to_owned());
        } else {
            self.validation_state.reject("client_id", VladError::MissingRequiredValue("client_id".to_owned()));
        }
        
        self.secret = try!(multimap_get_maybe_one(params, "secret")).map(|s| s.to_owned());
        self.hashed_secret = try!(multimap_get_maybe_one(params, "hashed_secret")).map(|s| s.to_owned());
        
        Ok(self.validation_state.valid)
    }
    
    pub fn build_from_params(params: &HashMap<String, Vec<String>>) -> Result<ClientApplication> {
        let mut builder = ClientApplicationBuilder::new();
        
        try!(builder.load_params(params));
        
        builder.build()
    }
}

pub trait ClientApplicationRepo where Self: Send + Sync {
    fn add_client_application(&self, u: ClientApplication) -> Result<()>;
    
    fn find_client_application(&self, client_id: &str) -> Result<Option<ClientApplication>>;
    
    fn update_client_application(&self, u: ClientApplication) -> Result<()>;
    
    fn remove_client_application(&self, client_id: &str) -> Result<()>;
}

#[derive(Clone)]
pub struct InMemoryClientApplicationRepo {
    client_applications: Arc<Mutex<Vec<ClientApplication>>>
}

impl InMemoryClientApplicationRepo {
    pub fn new() -> InMemoryClientApplicationRepo {
        InMemoryClientApplicationRepo {
            client_applications: Arc::new(Mutex::new(vec![])),
        }
    }
    
    fn get_index(entries: &Vec<ClientApplication>, client_id: &str) -> Result<usize> {
        Self::find_index(entries, client_id)
                .ok_or(OpenIdConnectError::ClientApplicationNotFound)
    }
    
    fn find_index(entries: &Vec<ClientApplication>, client_id: &str) -> Option<usize> {
        entries
                .iter()
                .position(|u| u.client_id == client_id)
    }
}

impl ClientApplicationRepo for InMemoryClientApplicationRepo {
    
    fn add_client_application(&self, ca: ClientApplication) -> Result<()> {
        let mut client_applications = self.client_applications.lock().unwrap();
        
        if Self::find_index(&client_applications, &ca.client_id).is_some() {
            Err(OpenIdConnectError::ClientApplicationAlreadyExists)
        } else {
            client_applications.push(ca);
            
            Ok(())
        }
    }
    
    fn find_client_application(&self, client_id: &str) -> Result<Option<ClientApplication>> {
        let client_applications = self.client_applications.lock().unwrap();
        
        Ok(Self::find_index(&client_applications, client_id).map(|i| client_applications[i].clone()))
    }
    
    fn update_client_application(&self, ca: ClientApplication) -> Result<()> {
        let mut client_applications = self.client_applications.lock().unwrap();
        
        let index = try!(Self::get_index(&client_applications, &ca.client_id));
        
        client_applications[index] = ca;
        
        Ok(())
    }
    
    fn remove_client_application(&self, client_id: &str) -> Result<()> {
        let mut client_applications = self.client_applications.lock().unwrap();
        
        let index = try!(Self::get_index(&client_applications, client_id));
        
        client_applications.remove(index);
        
        Ok(())
    }
}

impl Authenticator for InMemoryClientApplicationRepo {
    fn authenticate(&self, client_id: &str, secret: &str) -> Result<AuthenticationStatus> {
        let client_applications = self.client_applications.lock().unwrap();
        
        match client_applications.iter().find(|u| {
            u.client_id == client_id
        }) {
            Some(user) => {
                if user.hashed_secret == Some(hash_password(secret)) {
                    Ok(AuthenticationStatus::Success)
                } else {
                    Ok(AuthenticationStatus::IncorrectPassword)
                }
            },
            None => {
                Ok(AuthenticationStatus::PrincipalNotFound)
            }
        }
    }
}