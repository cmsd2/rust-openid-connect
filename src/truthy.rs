use std;
use result::OpenIdConnectError;

#[derive(Copy, Clone, Debug)]
pub enum Truthy {
    True,
    False
}

impl std::fmt::Display for Truthy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Truthy::True => write!(f, "True"),
            Truthy::False => write!(f, "False")
        }
    }
}

impl std::str::FromStr for Truthy {
    type Err = OpenIdConnectError;
    
    fn from_str(s: &str) -> std::result::Result<Truthy, OpenIdConnectError> {
        match s.chars().nth(0) {
            Some('t') => Ok(Truthy::True),
            Some('y') => Ok(Truthy::True),
            Some('T') => Ok(Truthy::True),
            Some('Y') => Ok(Truthy::True),
            Some('1') => Ok(Truthy::True),
            _ => Ok(Truthy::False), 
        }
    }
}

impl From<bool> for Truthy {
    fn from(b: bool) -> Truthy {
        if b {
            Truthy::True
        } else {
            Truthy::False
        }
    }
}

impl Into<bool> for Truthy {
    fn into(self) -> bool {
        match self {
            Truthy::True => true,
            Truthy::False => false
        }
    }
}