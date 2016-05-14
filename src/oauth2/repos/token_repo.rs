use std::sync::Arc;

use chrono::*;
use iron::prelude::*;
use oauth2::repos::GrantRepo;
use users::UserRepo;
use jsonwebtoken::jwt::*;
use jsonwebtoken::json::*;
use result::*;
use site_config::*;
use authentication::*;
use serialisation::*;

pub trait TokenRepo where Self: Send + Sync  {
    fn get_user_claims(&self, req: &mut Request, user_id: &str, client_id: &str, scopes: &[String]) -> Result<JwtClaims>;
}

pub struct InMemoryTokenRepo {
    user_repo: Arc<Box<UserRepo>>,
    grant_repo: Arc<Box<GrantRepo>>,
}

impl InMemoryTokenRepo {
    pub fn new(user_repo: Arc<Box<UserRepo>>, grant_repo: Arc<Box<GrantRepo>>) -> InMemoryTokenRepo {
        InMemoryTokenRepo {
            user_repo: user_repo,
            grant_repo: grant_repo,
        }
    }
}

impl TokenRepo for InMemoryTokenRepo {
    fn get_user_claims(&self, req: &mut Request, user_id: &str, client_id: &str, _scopes: &[String]) -> Result<JwtClaims> {
        let site_config = try!(SiteConfig::get(req));
        
        let maybe_user = try!(self.user_repo.get_user(user_id));
        let user = try!(maybe_user.ok_or(OpenIdConnectError::UserNotFound));
        
        let now = UTCDateTime::new(UTC::now());
        let later = UTCDateTime::new(try!(now.checked_add(site_config.get_token_duration()).ok_or(OpenIdConnectError::DateError)));
        
        let mut claims = JwtClaims::new();
        
        claims.set_value("iss", &site_config.get_issuer());
        claims.set_value("aud", &client_id);
        claims.set_value("sub", &user_id);
        claims.set_value("nonce", &new_nonce());
        claims.set_value("exp", &later);
        claims.set_value("nbf", &now);
        claims.set_value("iat", &now);
        claims.set_value("name", &user.username);
        //TODO more claims
        
        //TODO mask claims with granted permissions
        
        Ok(claims)
    }
}