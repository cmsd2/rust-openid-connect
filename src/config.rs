use std::sync::Arc;

use users::UserRepo;
use oauth2::ClientApplicationRepo;
use sessions::SessionController;

#[derive(Clone)]
pub struct Config
{
    pub user_repo: Arc<Box<UserRepo>>,
    pub application_repo: Arc<Box<ClientApplicationRepo>>,
    pub session_controller: SessionController,
}

impl Config {
    pub fn new(
            user_repo: Arc<Box<UserRepo>>, 
            application_repo: Arc<Box<ClientApplicationRepo>>,
            session_controller: SessionController) -> Config {
        Config {
            user_repo: user_repo,
            application_repo: application_repo,
            session_controller: session_controller,
        }
    }
}
