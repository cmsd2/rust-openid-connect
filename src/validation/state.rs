use std::collections::HashMap;

use super::result::ValidationError;
use super::field::Field;

pub trait State {
    fn accept(&mut self, field_name: &str);
    fn reject(&mut self, field_name: &str, reason: ValidationError);
}

#[derive(Clone, Debug)]
pub struct ValidationState
{
    pub valid: bool,
    pub fields: HashMap<String, Field>,
    pub errors: Vec<ValidationError>,
}

impl ValidationState {
    pub fn new() -> ValidationState {
        ValidationState {
            valid: true,
            fields: HashMap::new(),
            errors: vec![],
        }
    }
}

impl State for ValidationState {
    fn accept(&mut self, field_name: &str) {
        let mut field = self.fields.entry(field_name.to_owned()).or_insert(Field::new(field_name.to_owned(), true));
        field.valid = true;
    }
    
    fn reject(&mut self, field_name: &str, reason: ValidationError) {
        let mut field = self.fields.entry(field_name.to_owned()).or_insert(Field::new(field_name.to_owned(), false));
        field.valid = false;
        field.errors.push(reason);
        self.valid = false;
    }
}