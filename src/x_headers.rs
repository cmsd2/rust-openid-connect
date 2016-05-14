use std;
use std::fmt;
use iron::headers;
use iron::headers::parsing::from_one_raw_str;
use iron::error;

#[derive(Clone, Debug)]
pub struct XForwardedProto {
    pub forwarded_proto: String,
}

impl XForwardedProto {
    pub fn new<S>(proto: S) -> XForwardedProto where S: Into<String> {
        XForwardedProto {
            forwarded_proto: proto.into()
        }
    }
}

impl headers::Header for XForwardedProto {
    fn header_name() -> &'static str { "X-Forwarded-Proto" }
    
    fn parse_header(raw: &[Vec<u8>]) -> std::result::Result<XForwardedProto, error::HttpError> {
        from_one_raw_str(raw).map(|s: String| XForwardedProto::new(s) )
    }
}

impl headers::HeaderFormat for XForwardedProto {
    fn fmt_header(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        write!(f, "{}", self.forwarded_proto)
    }
}

#[derive(Clone, Debug)]
pub struct XForwardedFor {
    pub forwarded_for: String,
}

#[derive(Clone, Debug)]
pub struct XForwardedPort {
    pub forwarded_port: u16,
}

impl XForwardedPort {
    pub fn new(port: u16) -> XForwardedPort {
        XForwardedPort {
            forwarded_port: port
        }
    }
}

impl headers::Header for XForwardedPort {
    fn header_name() -> &'static str { "X-Forwarded-Port" }
    
    fn parse_header(raw: &[Vec<u8>]) -> std::result::Result<XForwardedPort, error::HttpError> {
        let s: String = try!(from_one_raw_str(raw));
        
        let u: u16 = try!(s.parse::<u16>().map_err(|_e| error::HttpError::Header));
        
        Ok(XForwardedPort::new(u))
    }
}

impl headers::HeaderFormat for XForwardedPort {
    fn fmt_header(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        write!(f, "{}", self.forwarded_port)
    }
}