use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use rand;
use rand::Rng;

use rustc_serialize::base64;
use rustc_serialize::base64::ToBase64;
use iron::prelude::*;
use oven::prelude::*;
use persistent;

use result::*;
use login::*;
use users::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSession {
    username: String,
    user_id: String,
    session_id: String,
    authenticated: bool,
}

impl UserSession {
    pub fn new(user_id: String, username: String, session_id: String) -> UserSession {
        UserSession {
            username: username,
            user_id: user_id,
            session_id: session_id,
            authenticated: false,
        }
    }
}

impl LoginSession for UserSession {
    fn get_id(&self) -> String {
        self.session_id.clone()
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
                sessions.insert(session.session_id.clone(), session.clone());
                
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
        let config = try!(req.get::<persistent::Read<LoginConfig>>());
                
        let session = match req.get_cookie(&config.cookie_base.name) {
            Some(c) if !c.value.is_empty() => {
                Some(c.value.clone())
            },
            _ => None,
        };

        Ok(session)
    }

    pub fn load_session(&self, req: &mut Request) -> Result<Login<UserSession>> {
        let config_arc = try!(req.get::<persistent::Read<LoginConfig>>());
        let config = (*config_arc).clone();
                
        let session = if let Some(session_id) = try!(self.load_session_id(req)) {
            try!(self.sessions.lookup(&session_id))
        } else {
            None
        };
        
        Ok(Login::new(&config, session))
    }
    
    pub fn clear_session(&self, req: &mut Request) -> Result<bool> { 
        if let Some(session_id) = try!(self.load_session_id(req)) {
            self.sessions.remove(&session_id)
        } else {
            Ok(false)
        }
    }
}
