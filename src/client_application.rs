use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use vlad::result::VladError;
use vlad::params::*;
use vlad::state::*;

use result::{Result, OpenIdConnectError};
use authentication::*;

#[derive(Clone,Debug)]
pub struct ClientApplication {
    pub access_key: String,
    pub secret: Option<String>,
    pub hashed_secret: Option<String>,
    pub redirect_uris: Vec<String>,
}

impl ClientApplication {
    pub fn new(access_key: String, secret: Option<String>) -> ClientApplication {
        ClientApplication {
            access_key: access_key,
            hashed_secret: Some(hash_password(secret.as_ref().map(|s| &s[..]).unwrap_or(""))),
            secret: secret,
            redirect_uris: vec![],
        }
    }
}

#[derive(Clone, Debug)]
pub struct ClientApplicationBuilder {
    access_key: Option<String>,
    secret: Option<String>,
    hashed_secret: Option<String>,
    redirect_uris: Vec<String>,
    
    validation_state: ValidationState,
}

impl ClientApplicationBuilder {
    pub fn new() -> ClientApplicationBuilder {
        ClientApplicationBuilder {
            access_key: None,
            secret: None,
            hashed_secret: None,
            redirect_uris: vec![],
            validation_state: ValidationState::new(),
        }
    }
    
    pub fn build(self) -> Result<ClientApplication> {
        Ok(ClientApplication {
            access_key: try!(self.access_key.ok_or(VladError::MissingRequiredValue("access_key".to_owned()))),
            secret: self.secret,
            hashed_secret: self.hashed_secret,
            redirect_uris: self.redirect_uris,
        })
    }
    
    pub fn load_params(&mut self, params: &HashMap<String, Vec<String>>) -> Result<bool> {
        if let Some(access_key) = try!(multimap_get_maybe_one(params, "access_key")) {
            self.access_key = Some(access_key.to_owned());
        } else {
            self.validation_state.reject("access_key", VladError::MissingRequiredValue("access_key".to_owned()));
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
    
    fn find_client_application(&self, access_key: &str) -> Result<Option<ClientApplication>>;
    
    fn update_client_application(&self, u: ClientApplication) -> Result<()>;
    
    fn remove_client_application(&self, access_key: &str) -> Result<()>;
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
    
    fn get_index(entries: &Vec<ClientApplication>, access_key: &str) -> Result<usize> {
        Self::find_index(entries, access_key)
                .ok_or(OpenIdConnectError::ClientApplicationNotFound)
    }
    
    fn find_index(entries: &Vec<ClientApplication>, access_key: &str) -> Option<usize> {
        entries
                .iter()
                .position(|u| u.access_key == access_key)
    }
}

impl ClientApplicationRepo for InMemoryClientApplicationRepo {
    
    fn add_client_application(&self, ca: ClientApplication) -> Result<()> {
        let mut client_applications = self.client_applications.lock().unwrap();
        
        if Self::find_index(&client_applications, &ca.access_key).is_some() {
            Err(OpenIdConnectError::ClientApplicationAlreadyExists)
        } else {
            client_applications.push(ca);
            
            Ok(())
        }
    }
    
    fn find_client_application(&self, access_key: &str) -> Result<Option<ClientApplication>> {
        let client_applications = self.client_applications.lock().unwrap();
        
        Ok(Self::find_index(&client_applications, access_key).map(|i| client_applications[i].clone()))
    }
    
    fn update_client_application(&self, ca: ClientApplication) -> Result<()> {
        let mut client_applications = self.client_applications.lock().unwrap();
        
        let index = try!(Self::get_index(&client_applications, &ca.access_key));
        
        client_applications[index] = ca;
        
        Ok(())
    }
    
    fn remove_client_application(&self, access_key: &str) -> Result<()> {
        let mut client_applications = self.client_applications.lock().unwrap();
        
        let index = try!(Self::get_index(&client_applications, access_key));
        
        client_applications.remove(index);
        
        Ok(())
    }
}

// TODO proper password hashing
pub fn hash_password(password: &str) -> String {
    password.to_owned()
}

impl Authenticator for InMemoryClientApplicationRepo {
    fn authenticate(&self, access_key: &str, secret: &str) -> Result<AuthenticationStatus> {
        let client_applications = self.client_applications.lock().unwrap();
        
        match client_applications.iter().find(|u| {
            u.access_key == access_key
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