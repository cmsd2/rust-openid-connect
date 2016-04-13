use std::result;
use result::{Result, OpenIdConnectError};

/// A way of authenticating users against a repository of users.

pub enum AuthenticationStatus {
    UserNotFound,
    IncorrectPassword,
    Success,
    //Continue, second factor?
}

pub trait Authenticator {
    fn authenticate(&self, username: &str, password: &str) -> Result<AuthenticationStatus>;
}

pub struct User {
    username: String,
    password: Option<String>,
    hashed_password: Option<String>,
}

pub struct InMemoryUserRepo {
    users: Vec<User>,
}

impl InMemoryUserRepo {
    pub fn new() -> InMemoryUserRepo {
        InMemoryUserRepo {
            users: vec![],
        }
    }
    
    pub fn add_user(&mut self, u: User) -> Result<()> {
        let exists = try!(self.find_user(&u.username)).is_some();
        
        if exists {
            Err(OpenIdConnectError::UserAlreadyExists)
        } else {
            self.users.push(u);
            Ok(())
        }
    }
    
    pub fn find_user<'a>(&'a self, username: &str) -> Result<Option<&'a User>> {
        Ok(self.users.iter().find(|u| {
            u.username == username
        }))
    }
    
    pub fn update_user(&mut self, u: User) -> Result<()> {
        let index = try!(self.get_index(&u.username));

        self.users[index] = u;
        
        Ok(())
    }
    
    pub fn remove_user(&mut self, username: &str) -> Result<()> {
        let index = try!(self.get_index(username));
        
        self.users.remove(index);
        
        Ok(())
    }
    
    fn get_index(&self, username: &str) -> Result<usize> {
        self.users
                .iter()
                .position(|u| u.username == username)
                .ok_or(OpenIdConnectError::UserNotFound)
    }
}

// TODO proper password hashing
pub fn hash_password(password: &str) -> String {
    password.to_owned()
}

impl Authenticator for InMemoryUserRepo {
    fn authenticate(&self, username: &str, password: &str) -> Result<AuthenticationStatus> {
        match self.users.iter().find(|u| {
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

