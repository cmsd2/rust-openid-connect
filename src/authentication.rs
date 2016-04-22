use result::{Result};
use rand::{Rng, thread_rng};
use rustc_serialize::base64;
use rustc_serialize::base64::ToBase64;

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

pub fn new_user_id() -> String {
    let mut bytes = [0u8, 12];
    thread_rng().fill_bytes(&mut bytes);
    bytes.to_base64(base64::URL_SAFE)
}

pub fn new_client_id() -> String {
    let mut bytes = [0u8; 12];
    thread_rng().fill_bytes(&mut bytes);
    bytes.to_base64(base64::URL_SAFE)
}

pub fn new_secret() -> String {
    let mut bytes = [0u8; 30];
    thread_rng().fill_bytes(&mut bytes);
    bytes.to_base64(base64::URL_SAFE)
}