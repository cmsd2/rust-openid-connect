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

