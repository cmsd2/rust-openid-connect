use std::sync::Arc;

use iron::prelude::*;
use iron::typemap;
use persistent;

use users::UserRepo;
use oauth2::ClientApplicationRepo;
use sessions::SessionController;
use result::*;
use jsonwebtoken::crypto::mac_signer::MacSigner;

#[derive(Clone)]
pub struct Config
{
    pub mac_signer: MacSigner,
    pub user_repo: Arc<Box<UserRepo>>,
    pub application_repo: Arc<Box<ClientApplicationRepo>>,
    pub session_controller: SessionController,
}

impl Config {
    pub fn new(
            mac_signer: MacSigner,
            user_repo: Arc<Box<UserRepo>>, 
            application_repo: Arc<Box<ClientApplicationRepo>>,
            session_controller: SessionController) -> Config {
        Config {
            mac_signer: mac_signer,
            user_repo: user_repo,
            application_repo: application_repo,
            session_controller: session_controller,
        }
    }
    
    pub fn get(req: &mut Request) -> Result<Arc<Config>> {
        req.get::<persistent::Read<Config>>().map_err(OpenIdConnectError::from)
    }
}

impl typemap::Key for Config {
    type Value = Config;
}
