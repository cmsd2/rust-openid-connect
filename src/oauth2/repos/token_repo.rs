use std::sync::Arc;
use std::sync::Mutex;

use chrono::*;
use iron::prelude::*;
use oauth2::repos::GrantRepo;
use users::UserRepo;
use jsonwebtoken::jwt::*;
use jsonwebtoken::json::*;
use jsonwebtoken::header::*;
use result::*;
use site_config::*;
use config::*;
use authentication;
use serialisation::*;
use oauth2::models::*;

pub trait TokenRepo where Self: Send + Sync  {
    fn get_user_claims(&self, req: &mut Request, user_id: &str, client_id: &str, scopes: &[String]) -> Result<JwtClaims>;
    
    fn create_auth_code(&self, req: &mut Request, user_id: &str, authorize_request: &AuthorizeRequest) -> Result<String>;
    
    fn create_token(&self, req: &mut Request, user_id: &str, authorize_request: &AuthorizeRequest) -> Result<Token>;

    fn exchange_auth_code(&self, req: &mut Request, code: &str) -> Result<Token>;
}

#[derive(Clone, Debug)]
pub struct AuthCode {
    pub code: String,
    pub exchanged: bool,
    pub created_at: DateTime<UTC>,
    pub expires_at: DateTime<UTC>,
}

impl AuthCode {
    pub fn new(code: String, created_at: DateTime<UTC>, expires_at: DateTime<UTC>) -> AuthCode {
        AuthCode {
            code: code,
            exchanged: false,
            created_at: created_at,
            expires_at: expires_at,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AuthEntry {
    pub code: AuthCode,
    pub user_id: String,
    pub authorize_request: AuthorizeRequest,
    pub token: Option<Token>,
    pub revoked: bool,
}

impl AuthEntry {
    pub fn new(user_id: String, code: AuthCode, authorize_request: AuthorizeRequest, token: Option<Token>) -> AuthEntry {
        AuthEntry {
            user_id: user_id,
            code: code,
            authorize_request: authorize_request,
            token: token,
            revoked: false,
        }
    }
}

pub struct InMemoryTokenRepo {
    user_repo: Arc<Box<UserRepo>>,
    grant_repo: Arc<Box<GrantRepo>>,
    auth_entries: Arc<Mutex<Vec<AuthEntry>>>,
}

impl InMemoryTokenRepo {
    pub fn new(user_repo: Arc<Box<UserRepo>>, grant_repo: Arc<Box<GrantRepo>>) -> InMemoryTokenRepo {
        InMemoryTokenRepo {
            user_repo: user_repo,
            grant_repo: grant_repo,
            auth_entries: Arc::new(Mutex::new(vec![])),
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
        claims.set_value("nonce", &authentication::new_nonce());
        claims.set_value("exp", &later);
        claims.set_value("nbf", &now);
        claims.set_value("iat", &now);
        claims.set_value("name", &user.username);
        //TODO more claims
        
        //TODO mask claims with granted permissions
        
        Ok(claims)
    }
    
    fn create_auth_code(&self, req: &mut Request, user_id: &str, authorize_request: &AuthorizeRequest) -> Result<String> {
        let site_config = try!(SiteConfig::get(req));
        let mut auth_entries = self.auth_entries.lock().unwrap();
        
        let now = UTC::now();
        let later = try!(now.checked_add(site_config.get_code_duration()).ok_or(OpenIdConnectError::DateError));
        
        let code = authentication::new_token();
        let auth_code = AuthCode::new(code.clone(), now, later);
        let auth_entry = AuthEntry::new(user_id.to_owned(), auth_code, authorize_request.to_owned(), None);
        
        auth_entries.push(auth_entry);
        
        Ok(code)
    }
    
    fn create_token(&self, req: &mut Request, user_id: &str, authorize_request: &AuthorizeRequest) -> Result<Token> {
        let config = try!(Config::get(req));
        let site_config = try!(SiteConfig::get(req));
        let claims = try!(self.get_user_claims(req, user_id, &authorize_request.client_id, &authorize_request.scopes));
        let id_token = Jwt::new(Header::default(), claims);
        let access_token = authentication::new_token();
        let refresh_token = Some(authentication::new_token());
        let expires_in = site_config.get_token_duration().into();
        let id_token = try!(id_token.encode(&config.mac_signer).map_err(OpenIdConnectError::from));
        
        let token = Token::new(access_token, refresh_token, expires_in, Some(id_token));
        
        Ok(token)
    }
    
    fn exchange_auth_code(&self, req: &mut Request, code: &str) -> Result<Token> {
        let mut auth_entries = self.auth_entries.lock().unwrap();
        
        let auth_entry = auth_entries
                .iter_mut()
                .find(|c| c.code.code == code);
        
        if let Some(auth_entry) = auth_entry {
            if auth_entry.code.exchanged == false && !auth_entry.revoked {
                auth_entry.code.exchanged = true;
            
                if auth_entry.token.is_none() {
                    let authorize_request = &auth_entry.authorize_request;
                         
                    auth_entry.token = Some(try!(self.create_token(req, &auth_entry.user_id, &authorize_request)));
                }
                
                Ok(auth_entry.token.as_ref().map(|t| t.to_owned()).unwrap())
            } else {
                // code used twice
                auth_entry.revoked = true;
                
                Err(OpenIdConnectError::AuthCodeError)
            }
        } else {
            // no such code
            Err(OpenIdConnectError::AuthCodeError)
        }
    }
}