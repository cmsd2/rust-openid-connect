use std::sync::Arc;

use users::UserRepo;

#[derive(Clone)]
pub struct Config
{
    pub user_repo: Arc<Box<UserRepo>>,
}

impl Config {
    pub fn new(user_repo: Arc<Box<UserRepo>>) -> Config {
        Config {
            user_repo: user_repo
        }
    }
}
