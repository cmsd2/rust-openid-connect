use std::sync::Arc;

use iron::prelude::*;
use iron::typemap;
use persistent;

use site_config::*;
use users::UserRepo;
use oauth2::{ClientApplicationRepo, GrantRepo, TokenRepo};
use sessions::SessionController;
use result::*;
use jsonwebtoken::crypto::mac_signer::MacSigner;

#[derive(Clone)]
pub struct Config
{
    pub mac_signer: MacSigner,
    pub user_repo: Arc<Box<UserRepo>>,
    pub application_repo: Arc<Box<ClientApplicationRepo>>,
    pub grant_repo: Arc<Box<GrantRepo>>,
    pub token_repo: Arc<Box<TokenRepo>>,
    pub session_controller: SessionController,
    pub site_config: SiteConfig,
}

impl Config {
    pub fn new(
            mac_signer: MacSigner,
            user_repo: Arc<Box<UserRepo>>, 
            application_repo: Arc<Box<ClientApplicationRepo>>,
            grant_repo: Arc<Box<GrantRepo>>,
            token_repo: Arc<Box<TokenRepo>>,
            session_controller: SessionController) -> Config {
        Config {
            mac_signer: mac_signer,
            user_repo: user_repo,
            application_repo: application_repo,
            grant_repo: grant_repo,
            token_repo: token_repo,
            session_controller: session_controller,
            site_config: SiteConfig::default(),
        }
    }
    
    pub fn get(req: &mut Request) -> Result<Arc<Config>> {
        req.get::<persistent::Read<Config>>().map_err(OpenIdConnectError::from)
    }
}

impl typemap::Key for Config {
    type Value = Config;
}
