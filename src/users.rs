use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use validation::result::ValidationError;
use validation::params::*;
use validation::state::*;

use result::{Result, OpenIdConnectError};
use authentication::*;

#[derive(Clone,Debug)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: Option<String>,
    pub hashed_password: Option<String>,
}

impl User {
    pub fn new(id: String, username: String, password: Option<String>) -> User {
        User {
            id: id,
            username: username,
            hashed_password: Some(hash_password(password.as_ref().map(|s| &s[..]).unwrap_or(""))),
            password: password,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UserBuilder {
    pub username: Option<String>,
    pub password: Option<String>,
    pub hashed_password: Option<String>,
    
    validation_state: ValidationState,
}

impl UserBuilder {
    pub fn new() -> UserBuilder {
        UserBuilder {
            username: None,
            password: None,
            hashed_password: None,
            validation_state: ValidationState::new(),
        }
    }
    
    pub fn build(self) -> Result<User> {
        Ok(User {
            id: new_user_id(),
            username: try!(self.username.ok_or(ValidationError::MissingRequiredValue("username".to_owned()))),
            password: self.password,
            hashed_password: self.hashed_password,
        })
    }
    
    pub fn load_params(&mut self, params: &HashMap<String, Vec<String>>) -> Result<bool> {
        if let Some(username) = try!(multimap_get_maybe_one(params, "username")) {
            self.username = Some(username.to_owned());
        } else {
            self.validation_state.reject("username", ValidationError::MissingRequiredValue("username".to_owned()));
        }
        
        self.password = try!(multimap_get_maybe_one(params, "password")).map(|s| s.to_owned());
        self.hashed_password = try!(multimap_get_maybe_one(params, "hashed_password")).map(|s| s.to_owned());
        
        Ok(self.validation_state.valid)
    }
    
    pub fn build_from_params(params: &HashMap<String, Vec<String>>) -> Result<User> {
        let mut builder = UserBuilder::new();
        
        try!(builder.load_params(params));
        
        builder.build()
    }
}

pub trait UserRepo where Self: Send + Sync {
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
                Ok(AuthenticationStatus::PrincipalNotFound)
            }
        }
    }
}