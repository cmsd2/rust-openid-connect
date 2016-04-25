use std::collections::HashMap;

use frank_jwt::Header;
use frank_jwt::Payload;
use frank_jwt::encode;
use frank_jwt::decode;
use frank_jwt::Algorithm;

use config::Config;

pub fn new_token(config: &Config, claims: HashMap<String, String>) -> String {
    let mut payload = Payload::new();
    
    for (k,v) in claims {
        payload.insert(k, v);
    }

    // TODO get secret from somewhere safe
    let secret = "secret123";
    
    // TODO allow selection of algorithm
    let header = Header::new(Algorithm::HS256);

    encode(header, secret.to_string(), payload)
}