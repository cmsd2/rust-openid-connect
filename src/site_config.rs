use std;
use std::borrow::Cow;
use std::sync::Arc;
use std::ops::Deref;

use serde;
use chrono::*;
use iron::prelude::*;
use iron::typemap;
use iron::Url;
use persistent;

use result;
use result::{OpenIdConnectError};
use serialisation::*;
use grant_type::*;

#[derive(Clone, Debug)]
pub struct SiteUrl {
    pub url: Url
}

impl SiteUrl {
    pub fn new(url: Url) -> SiteUrl {
        SiteUrl {
            url: url
        }
    }
}

impl Deref for SiteUrl {
    type Target = Url;
    fn deref(&self) -> &Url {
        &self.url
    }
}

impl Into<Url> for SiteUrl {
    fn into(self) -> Url {
        self.url
    }
}

impl From<Url> for SiteUrl {
    fn from(url: Url) -> SiteUrl {
        SiteUrl::new(url)
    }
}

impl serde::ser::Serialize for SiteUrl {
        fn serialize<S>(&self, serializer: &mut S) -> std::result::Result<(), S::Error>
        where S: serde::Serializer,
    {
        self.url.serialize_with(serializer)
    }
}

impl serde::de::Deserialize for SiteUrl {
        fn deserialize<D>(deserializer: &mut D) -> std::result::Result<SiteUrl, D::Error>
        where D: serde::de::Deserializer
    {
        let url: Url = try!(DeserializeWith::deserialize_with(deserializer));
        
        Ok(SiteUrl::new(url))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TokenDuration {
    pub duration: Duration,
}

impl TokenDuration {
    pub fn new(duration: Duration) -> TokenDuration {
        TokenDuration {
            duration: duration,
        }
    }
}

impl Into<Duration> for TokenDuration {
    fn into(self) -> Duration {
        self.duration
    }
}

impl From<Duration> for TokenDuration {
    fn from(d: Duration) -> TokenDuration {
        TokenDuration::new(d)
    }
}

impl Deref for TokenDuration {
    type Target = Duration;
    fn deref(&self) -> &Duration {
        &self.duration
    }
}

impl serde::ser::Serialize for TokenDuration {
        fn serialize<S>(&self, serializer: &mut S) -> std::result::Result<(), S::Error>
        where S: serde::Serializer,
    {
        self.duration.serialize_with(serializer)
    }
}

impl serde::de::Deserialize for TokenDuration {
        fn deserialize<D>(deserializer: &mut D) -> std::result::Result<TokenDuration, D::Error>
        where D: serde::de::Deserializer
    {
        let duration: Duration = try!(DeserializeWith::deserialize_with(deserializer));
        
        Ok(TokenDuration::new(duration))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SiteConfig {
    pub token_issuer: Option<String>, // token iss claim
    pub token_duration: Option<TokenDuration>,
    pub code_duration: Option<TokenDuration>,
    pub listen_host: Option<String>, // ip address to listen on. default 0.0.0.0
    pub listen_port: Option<u16>, // port to listen on. default 8080
    pub base_url: Option<SiteUrl>, // default base url for constructing absolute urls
    pub use_x_forwarded_proto: bool, // override base_url protocol with x-forwarded-proto header
    pub use_x_forwarded_port: bool, // override base_url port with x-forwarded-port header
    // other stuff like key sets, timeouts, policies etc
    pub enable_oauth2: bool, // access tokens and refresh tokens without openid. if false, scope must include "openid".
    pub enabled_grants: Vec<GrantType>, // permitted grant types (authorization_code, client_credentials, ...)
}

impl Default for SiteConfig {
    fn default() -> SiteConfig {
        SiteConfig {
            token_issuer: None,
            token_duration: None,
            code_duration: None,
            listen_host: None,
            listen_port: None,
            base_url: None,
            use_x_forwarded_proto: true,
            use_x_forwarded_port: true,
            enable_oauth2: true,
            enabled_grants: vec![GrantType::AuthorizationCode, GrantType::ClientCredentials],
        }
    }
}

impl SiteConfig {
    pub fn new() -> SiteConfig {
        SiteConfig::default()
    }
    
    pub fn get_listen_host<'a>(&'a self) -> Cow<'a, String> {
        self.listen_host.as_ref().map(|s| Cow::Borrowed(s)).unwrap_or(Cow::Owned("0.0.0.0".to_owned()))
    }
    
    pub fn get_listen_port(&self) -> u16 {
        self.listen_port.unwrap_or(8080)
    }
    
    pub fn get_listen_host_port(&self) -> String {
        format!("{}:{}", self.get_listen_host(), self.get_listen_port())
    }
    
    pub fn get_listen_url(&self) -> String {
        if self.get_listen_port() == 80 {
            format!("http://{}", self.get_listen_host())
        } else {
            format!("http://{}:{}", self.get_listen_host(), self.get_listen_port())
        }
    }
    
    pub fn get_issuer(&self) -> String {
        self.token_issuer.as_ref().map(|s| s.to_owned()).unwrap_or(self.get_listen_url())
    }
    
    pub fn get_token_duration(&self) -> Duration {
        self.token_duration.as_ref().map(|d| d.to_owned()).unwrap_or(Duration::hours(24).into()).into()
    }
    
    pub fn get_code_duration(&self) -> Duration {
        self.code_duration.as_ref().map(|d| d.to_owned()).unwrap_or(Duration::minutes(10).into()).into()
    }
    
    pub fn get(req: &mut Request) -> result::Result<Arc<SiteConfig>> {
        req.get::<persistent::Read<SiteConfig>>().map_err(OpenIdConnectError::from)
    }
    
    pub fn grant_enabled(&self, grant_type: GrantType) -> bool {
        self.enabled_grants.contains(&grant_type)
    }
}

impl typemap::Key for SiteConfig {
    type Value = SiteConfig;
}