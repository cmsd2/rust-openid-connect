use result::{Result};
/// A way of authenticating users against a repository of users.

pub enum AuthenticationStatus {
    PrincipalNotFound,
    IncorrectPassword,
    Success,
    //Continue, second factor?
}

pub trait Authenticator {
    fn authenticate(&self, username: &str, password: &str) -> Result<AuthenticationStatus>;
}

// TODO proper password hashing
pub fn hash_password(password: &str) -> String {
    password.to_owned()
}