use std::sync::{Arc, Mutex};

use result::{Result, OpenIdConnectError};
use authentication::*;

#[derive(Clone,Debug)]
pub struct User {
    pub username: String,
    pub password: Option<String>,
    pub hashed_password: Option<String>,
}

pub trait UserRepo {
    fn add_user(&self, u: User) -> Result<()>;
    
    fn find_user(&self, username: &str) -> Result<Option<User>>;
    
    fn update_user(&self, u: User) -> Result<()>;
    
    fn remove_user(&self, username: &str) -> Result<()>;
}

#[derive(Clone)]
pub struct InMemoryUserRepo {
    users: Arc<Mutex<Vec<User>>>
}

impl InMemoryUserRepo {
    pub fn new() -> InMemoryUserRepo {
        InMemoryUserRepo {
            users: Arc::new(Mutex::new(vec![])),
        }
    }
    
    fn get_index(users: &Vec<User>, username: &str) -> Result<usize> {
        Self::find_index(users, username)
                .ok_or(OpenIdConnectError::UserNotFound)
    }
    
    fn find_index(users: &Vec<User>, username: &str) -> Option<usize> {
        users
                .iter()
                .position(|u| u.username == username)
    }
}

impl UserRepo for InMemoryUserRepo {
    
    fn add_user(&self, u: User) -> Result<()> {
        let mut users = self.users.lock().unwrap();
        
        if Self::find_index(&users, &u.username).is_some() {
            Err(OpenIdConnectError::UserAlreadyExists)
        } else {
            users.push(u);
            
            Ok(())
        }
    }
    
    fn find_user(&self, username: &str) -> Result<Option<User>> {
        let users = self.users.lock().unwrap();
        
        Ok(Self::find_index(&users, username).map(|i| users[i].clone()))
    }
    
    fn update_user(&self, u: User) -> Result<()> {
        let mut users = self.users.lock().unwrap();
        
        let index = try!(Self::get_index(&users, &u.username));
        
        users[index] = u;
        
        Ok(())
    }
    
    fn remove_user(&self, username: &str) -> Result<()> {
        let mut users = self.users.lock().unwrap();
        
        let index = try!(Self::get_index(&users, username));
        
        users.remove(index);
        
        Ok(())
    }
}

// TODO proper password hashing
pub fn hash_password(password: &str) -> String {
    password.to_owned()
}

impl Authenticator for InMemoryUserRepo {
    fn authenticate(&self, username: &str, password: &str) -> Result<AuthenticationStatus> {
        let users = self.users.lock().unwrap();
        
        match users.iter().find(|u| {
            u.username == username
        }) {
            Some(user) => {
                if user.hashed_password == Some(hash_password(password)) {
                    Ok(AuthenticationStatus::Success)
                } else {
                    Ok(AuthenticationStatus::IncorrectPassword)
                }
            },
            None => {
                Ok(AuthenticationStatus::UserNotFound)
            }
        }
    }
}