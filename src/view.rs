use std::collections::HashMap;
use sessions::UserSession;
use iron::prelude::*;
use result::Result;
use handlebars_iron::Template;
use serde_json::value::{self, Value};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct View {
    pub data: HashMap<String, Value>,
    pub session: Option<UserSession>,
    pub view: String,
    pub layout: Option<String>,
    pub csrf_token: Option<String>,
}

impl View {
    pub fn new<S: Into<String>>(name: S, session: Option<UserSession>) -> View {
        View {
            view: name.into(),
            data: HashMap::new(),
            session: session,
            csrf_token: None,
            layout: Some("_layout.html".to_owned()),
        }
    }
    
    pub fn new_for_session(name: &str, req: &mut Request) -> Result<View> {
        let session = try!(req.get::<UserSession>());
        
        Ok(View::new(name, session))
    }
    
    pub fn template(self) -> Result<Template> {
        let template_name = if let Some(layout) = self.layout.clone() {
            layout
        } else {
            self.view.clone()
        };
        
        let mut data = self.data;
        
        data.insert("view".to_owned(), try!(value::to_value(&self.view)));
        data.insert("session".to_owned(), try!(value::to_value(&self.session)));
        
        if let Some(csrf_token) = self.csrf_token {
            data.insert("csrf_token".to_owned(), try!(value::to_value(&csrf_token)));
        }
        
        debug!("rendering view {} {:?}", template_name, data);
        
        Ok(Template::new(&template_name, data))
    }
}