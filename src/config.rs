use std::sync::Arc;

use users::UserRepo;
use client_application::ClientApplicationRepo;

#[derive(Clone)]
pub struct Config
{
    pub user_repo: Arc<Box<UserRepo>>,
    pub application_repo: Arc<Box<ClientApplicationRepo>>,
}

impl Config {
    pub fn new(user_repo: Arc<Box<UserRepo>>, application_repo: Arc<Box<ClientApplicationRepo>>) -> Config {
        Config {
            user_repo: user_repo,
            application_repo: application_repo,
        }
    }
}
