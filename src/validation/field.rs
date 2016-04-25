use super::result::ValidationError;

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

impl Field {
    pub fn new(name: String, valid: bool) -> Field {
        Field {
            name: name,
            valid: valid,
            errors: vec![]
        }
    }
}
