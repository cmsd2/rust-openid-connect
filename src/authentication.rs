use std::result;

/// A way of authenticating users against a repository of users.

quick_error! {
    #[derive(Debug)]
    pub enum AuthenticationError {
        UserNotFound {
            description("user not found")
            display("User not found")
        }
        
        IncorrectPassword {
            description("incorrect password")
            display("Incorrect password")
        }
    }
}

pub type AuthenticationResult<T> = result::Result<T, AuthenticationError>;

pub trait UserRepo {
    fn authenticate(username: &str, password: &str) -> AuthenticationResult<()>;
}