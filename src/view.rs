use std::collections::HashMap;
use sessions::UserSession;
use iron::prelude::*;
use result::Result;
use handlebars_iron::Template;
use serde_json::value::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct View {
    pub data: HashMap<String, Value>,
    pub session: Option<UserSession>,
    pub view: String,
    pub layout: Option<String>,
}

impl View {
    pub fn new<S: Into<String>>(name: S, session: Option<UserSession>) -> View {
        View {
            view: name.into(),
            data: HashMap::new(),
            session: session,
            layout: Some("_layout.html".to_owned()),
        }
    }
    
    pub fn new_for_session(name: &str, req: &mut Request) -> Result<View> {
        let session = try!(req.get::<UserSession>());
        
        Ok(View::new(name, session))
    }
    
    pub fn template(self) -> Template {
        let template_name = if let Some(layout) = self.layout.clone() {
            layout
        } else {
            self.view.clone()
        };
        
        debug!("rendering view {} {:?}", template_name, self);
        
        Template::new(&template_name, self)
    }
}