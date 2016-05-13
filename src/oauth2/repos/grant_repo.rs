use std::sync::{Arc, Mutex};

use chrono::*;

use result::{Result, OpenIdConnectError};
use oauth2::models::grant::*;

pub trait GrantRepo where Self: Send + Sync {
    fn get_user_grants(&self, user_id: &str) -> Result<Vec<Grant>>;
    
    fn create_or_update_grant(&self, ca: GrantUpdate) -> Result<Grant>;
      
    fn find_grant(&self, user_id: &str, client_id: &str) -> Result<Option<Grant>>;
    
    fn touch_grant(&self, g: &Grant) -> Result<()>;
    
    fn remove_grant(&self, user_id: &str, client_id: &str) -> Result<()>;
}

#[derive(Clone)]
pub struct InMemoryGrantRepo {
    grants: Arc<Mutex<Vec<Grant>>>
}

impl InMemoryGrantRepo {
    pub fn new() -> InMemoryGrantRepo {
        InMemoryGrantRepo {
            grants: Arc::new(Mutex::new(vec![])),
        }
    }
    
    fn get_index(entries: &Vec<Grant>, user_id: &str, client_id: &str) -> Result<usize> {
        Self::find_index(entries, user_id, client_id)
                .ok_or(OpenIdConnectError::GrantNotFound)
    }
    
    fn find_index(entries: &Vec<Grant>, user_id: &str, client_id: &str) -> Option<usize> {
        entries
                .iter()
                .position(|u| u.user_id == user_id && u.client_id == client_id)
    }
}

impl GrantRepo for InMemoryGrantRepo {
    fn get_user_grants(&self, user_id: &str) -> Result<Vec<Grant>> {
        debug!("grants: select * for user {}", user_id);
        
        let grants = self.grants.lock().unwrap();
        
        Ok(grants.iter().filter(|g| g.user_id == user_id).map(|g| g.clone()).collect())
    }
    
    fn create_or_update_grant(&self, input: GrantUpdate) -> Result<Grant> {
        debug!("grant: create {:?}", input);
        
        let mut grants = self.grants.lock().unwrap();
        
        if let Some(index) = Self::find_index(&grants, &input.user_id, &input.client_id) {
            grants[index].update(input);
            Ok(grants[index].clone())
        } else {
            grants.push(Grant::new_for_update(input));
            Ok(grants[grants.len() -1].clone())
        }
    }
    
    fn find_grant(&self, user_id: &str, client_id: &str) -> Result<Option<Grant>> {
        debug!("grants: find for user {} and client {}", user_id, client_id);
        
        let grants = self.grants.lock().unwrap();
        
        Ok(Self::find_index(&grants, user_id, client_id).map(|i| grants[i].clone()))
    }
    
    fn touch_grant(&self, g: &Grant) -> Result<()> {
        debug!("grants: touch {:?}", g);
        
        let mut grants = self.grants.lock().unwrap();
        
        let index = try!(Self::get_index(&grants, &g.user_id, &g.client_id));
        
        grants[index].accessed_at = UTC::now();
        
        Ok(())
    }
    
    fn remove_grant(&self, user_id: &str, client_id: &str) -> Result<()> {
        debug!("grants: delete for user_id {} client_id {}", user_id, client_id);
        
        let mut grants = self.grants.lock().unwrap();
        
        let index = try!(Self::get_index(&grants, user_id, client_id));
        
        grants.remove(index);
        
        Ok(())
    }
}
