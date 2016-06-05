use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use std::borrow::Cow;
use rand;
use rand::Rng;
use crypto::digest::Digest;
use crypto::md5::Md5;

use rustc_serialize::base64;
use rustc_serialize::base64::ToBase64;
use iron::prelude::*;
use iron::BeforeMiddleware;
use iron::typemap::Key;
use oven::prelude::*;
use persistent;
use plugin;
use plugin::Extensible;
use urlencoded::*;

use result::*;
use login_manager::*;
use users::*;
use rbvt::params::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
}

impl Credentials {
    pub fn new<A: Into<String>, B: Into<String>>(username: A, password: B) -> Credentials {
        Credentials {
            username: username.into(),
            password: password.into(),
        }
    }
}

pub struct Gravatar;

impl Gravatar {
    pub fn hash(email: &str) -> String {
        let s = email.to_owned().trim().to_lowercase();
        let mut md5 = Md5::new();
        md5.input_str(&s);
        md5.result_str().to_owned()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub username: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub authenticated: bool,
    pub gravatar: Option<String>,
}

impl UserSession {
    pub fn new(user_id: String, username: String, session_id: String) -> UserSession {
        UserSession {
            gravatar: Some(Gravatar::hash(&username)),
            username: Some(username),
            user_id: Some(user_id),
            session_id: Some(session_id),
            authenticated: false,
        }
    }
}

impl Default for UserSession {
    fn default() -> UserSession {
        UserSession {
            username: None,
            user_id: None,
            session_id: None,
            authenticated: false,
            gravatar: None,
        }
    }
}

impl LoginSession for UserSession {
    fn get_id(&self) -> Option<String> {
        self.session_id.clone()
    }
}

pub type SessionLogin = Login<UserSession>;

impl Key for UserSession { type Value = Option<UserSession>; }

impl<'a, 'b> plugin::Plugin<Request<'a, 'b>> for UserSession {
    type Error = OpenIdConnectError;
    
    fn eval(req: &mut Request) -> Result<Option<UserSession>> {
        debug!("getting session from middleware chain");
        req.extensions().get::<UserSession>().ok_or(OpenIdConnectError::NoSessionLoaded).map(|s| s.to_owned())
    }
}


pub trait Sessions: Send + Sync + 'static {
    fn authenticate(&self, creds: &Credentials) -> Result<UserSession>;
    
    fn lookup(&self, session_id: &str) -> Result<Option<UserSession>>;
    
    fn remove(&self, session_id: &str) -> Result<bool>;
}

pub struct InMemorySessions {
    users: Arc<Box<UserRepo>>,
    sessions: Arc<Mutex<HashMap<String, UserSession>>>,
}

impl InMemorySessions {
    pub fn new(users: Arc<Box<UserRepo>>) -> InMemorySessions {
        InMemorySessions {
            users: users,
            sessions: Arc::new(Mutex::new(HashMap::new()))
        }
    }
    
    fn new_session_id(&self) -> String {
        let mut id = vec![0u8; 16];
        rand::thread_rng().fill_bytes(id.as_mut_slice());
        id.to_base64(base64::STANDARD)
    }
    
    fn new_session(&self, user_id: &str, username: &str) -> UserSession {
        UserSession::new(user_id.to_owned(), username.to_owned(), self.new_session_id())
    }
}

impl Sessions for InMemorySessions {
    
    fn authenticate(&self, creds: &Credentials) -> Result<UserSession> {
        if let Some(user) = try!(self.users.find_user(&creds.username)) {
            if user.password.as_ref() == Some(&creds.password) {
                let mut session = self.new_session(&user.id, &user.username);
                session.authenticated = true;
                
                let mut sessions = self.sessions.lock().unwrap();
                let session_id = try!(session.session_id.as_ref().ok_or(OpenIdConnectError::InvalidUsernameOrPassword)).to_owned();
                sessions.insert(session_id, session.clone());
                
                Ok(session)
            } else {
                // TODO add random wait jitter
                Err(OpenIdConnectError::InvalidUsernameOrPassword)
            }
        } else {
            // TODO add random wait jitter
            Err(OpenIdConnectError::InvalidUsernameOrPassword)
        }
    }
    
    fn lookup(&self, session_id: &str) -> Result<Option<UserSession>> {
        let sessions = self.sessions.lock().unwrap();
        
        Ok(sessions.get(session_id).map(|u| (*u).clone()))
    }
    
    fn remove(&self, session_id: &str) -> Result<bool> {
        let mut sessions = self.sessions.lock().unwrap();
        
        Ok(sessions.remove(session_id).is_some())
    }
}

#[derive(Clone)]
pub struct SessionController {
    pub sessions: Arc<Box<Sessions>>,
    pub login_manager: LoginManager,
}

impl SessionController {
    pub fn new(sessions: Arc<Box<Sessions>>, login_manager: LoginManager) -> Self {
        SessionController {
            sessions: sessions,
            login_manager: login_manager,
        }
    }
    
    pub fn load_session_id(&self, req: &mut Request) -> Result<Option<String>> {
        debug!("loading session id");
        let config = try!(req.get::<persistent::Read<LoginConfig>>());
                
        let session = match req.get_cookie(&config.cookie_base.name) {
            Some(c) if !c.value.is_empty() => {
                Some(c.value.clone())
            },
            _ => None,
        };

        Ok(session)
    }

    pub fn load_session(&self, req: &mut Request) -> Result<Option<UserSession>> {
        debug!("loading session");
    
        let session = if let Some(session_id) = try!(self.load_session_id(req)) {
            debug!("looking up session {}", session_id);
            try!(self.sessions.lookup(&session_id))
        } else {
            None
        };
        
        Ok(session)
    }
    
    pub fn clear_session(&self, req: &mut Request) -> Result<bool> {
        debug!("clearing session");
        if let Some(session_id) = try!(self.load_session_id(req)) {
            self.sessions.remove(&session_id)
        } else {
            Ok(false)
        }
    }
    
    /// Login with credentials if provided.
    /// Separate from default login flow: only /login should all this.
    pub fn login_with_credentials(&self, req: &mut Request) -> Result<Login<UserSession>> {
        debug!("logging in with credentials");
        
        let login_config = try!(LoginConfig::get_config(req));
        
        let params = try!(match req.get_ref::<UrlEncodedBody>() {
            Ok(params) => Ok(Cow::Borrowed(params)),
            Err(UrlDecodingError::EmptyQuery) => Ok(Cow::Owned(HashMap::new())),
            Err(e) => Err(e),
        });
        
        // TODO validate csrf
        
        let username = try!(multimap_get_maybe_one(&params, "username").map_err(|e| {
            debug!("error reading username: {:?}", e);
            OpenIdConnectError::InvalidUsernameOrPassword
        }));
        
        let password = try!(multimap_get_maybe_one(&params, "password").map_err(|e| {
            debug!("error reading password: {:?}", e);
            OpenIdConnectError::InvalidUsernameOrPassword
        }));
        
        let session = if username.is_some() || password.is_some() {
            let creds = Credentials::new(username.unwrap_or(""), password.unwrap_or(""));
            
            let session = try!(self.sessions.authenticate(&creds));
            
            Some(session)
        } else {
            None
        };
        
        debug!("session: {:?}", session);

        let login_modifier = Login::new(&login_config, session);

        Ok(login_modifier)
    }
    
    /// Login with cookie if possible
    /// otherwise leave session blank
    pub fn login(&self, req: &mut Request) -> Result<Login<UserSession>> {

        let session = try!(self.load_session(req));
        
        debug!("session: {:?}", session);

        let login_config = try!(LoginConfig::get_config(req));
        let login_modifier = Login::new(&login_config, session);

        Ok(login_modifier)
    }
}

impl BeforeMiddleware for SessionController {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        match self.login(req) {
            Ok(login) => {
                debug!("injecting session into middleware chain {:?}", login);
                req.extensions_mut().insert::<UserSession>(login.session);
                Ok(())
            },
            Err(OpenIdConnectError::PersistentError(persistent::PersistentError::NotFound)) => {
                debug!("no session found");
                req.extensions_mut().insert::<UserSession>(None);
                Ok(())
            }
            Err(e) => {
                req.extensions_mut().insert::<UserSession>(None);
                Err(IronError::from(OpenIdConnectError::from(e)))
            }
        }
    }
}
