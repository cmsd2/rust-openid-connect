use std::sync::{Arc, Mutex};

use rbvt::result::ValidationError;

use result::{Result, OpenIdConnectError};
use authentication::*;

use super::super::models::client::*;

pub trait ClientApplicationRepo where Self: Send + Sync {
    fn create_client_application(&self, ca: ClientApplicationBuilder) -> Result<ClientApplication>;
    
    fn get_client_applications(&self) -> Result<Vec<ClientApplication>>;
    
    fn find_client_application(&self, client_id: &str) -> Result<Option<ClientApplication>>;
    
    fn update_client_application(&self, u: &ClientApplication) -> Result<()>;
    
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
    fn get_client_applications(&self) -> Result<Vec<ClientApplication>> {
        debug!("client_applications: select *");
        
        let client_applications = self.client_applications.lock().unwrap();
        
        Ok(client_applications.clone())
    }
    
    fn create_client_application(&self, mut input: ClientApplicationBuilder) -> Result<ClientApplication> {
        debug!("client_applications: create {:?}", input);
        
        let mut client_applications = self.client_applications.lock().unwrap();
        
        input.client_id = input.client_id.or_else(|| Some(new_client_id()));
        input.secret = Some(new_secret());
        input.hashed_secret = Some(hash_password(input.secret.as_ref().map(|s| &s[..]).unwrap_or("")));
        
        if ! try!(input.validate()) {  
            return Err(OpenIdConnectError::ValidationError(ValidationError::ValidationError(input.validation_state)));
        }
        
        let ca = try!(input.build());
        
        if Self::find_index(&client_applications, &ca.client_id).is_some() {
            Err(OpenIdConnectError::ClientApplicationAlreadyExists)
        } else {
            client_applications.push(ca.clone());
            
            Ok(ca)
        }
    }
    
    fn find_client_application(&self, client_id: &str) -> Result<Option<ClientApplication>> {
        debug!("client_applications: find {}", client_id);
        
        let client_applications = self.client_applications.lock().unwrap();
        
        Ok(Self::find_index(&client_applications, client_id).map(|i| client_applications[i].clone()))
    }
    
    fn update_client_application(&self, ca: &ClientApplication) -> Result<()> {
        debug!("client_applications: update {:?}", ca);
        
        let mut client_applications = self.client_applications.lock().unwrap();
        
        let index = try!(Self::get_index(&client_applications, &ca.client_id));
        
        client_applications[index] = ca.clone();
        
        Ok(())
    }
    
    fn remove_client_application(&self, client_id: &str) -> Result<()> {
        debug!("client_applications: delete {}", client_id);
        
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