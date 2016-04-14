
#[derive(Clone, Debug)]
pub struct User {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl User {
    pub fn new() -> User {
        User {
            username: None,
            password: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ValidatedUser {
    pub username: String,
    pub password: String,
}

impl ValidatedUser {
    pub fn new(username: String, password: String) -> ValidatedUser {
        ValidatedUser {
            username: username,
            password: password,
        }
    }
}