use std::result;
use std::collections::HashMap;
use std::io;

quick_error! {
    /// only for critical failures in validation,
    /// the validation result (whether yes or no)
    /// should be return in an Ok.
    #[derive(Debug)]
    pub enum ValidationError {
        IoError(err: io::Error) {
            from()
            description("io error")
            display("I/O error: {}", err)
            cause(err)
        }
    }
}

pub type ValidationResult<T> = result::Result<T, ValidationError>;

pub trait Validator<M> {
    fn validate(&mut self, model: &M) -> ValidationResult<bool>;
}

pub trait State {
    fn accept(&mut self, field_name: &str);
    fn reject(&mut self, field_name: &str, reason: String);
}

pub trait Rule<T, S>
{
	fn validate(&self, input:&T, state: &mut S) -> ValidationResult<()>;
}

impl <T, S, F> Rule<T, S> for F
where F: Fn(&T, &mut S) -> ValidationResult<()>
{
    fn validate(&self, input:&T, state: &mut S) -> ValidationResult<()> {
        (*self)(input, state)
    }
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub valid: bool,
    pub errors: Vec<String>,
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

pub struct ValidatorSchema<M> {
    pub state: ValidatorState,
    pub rules: Vec<Box<Rule<M, ValidatorState>>>
}

impl <M> ValidatorSchema<M> {
    pub fn new() -> Self {
        ValidatorSchema {
            state: ValidatorState::new(),
            rules: vec![],
        }
    }
    
    pub fn rule(&mut self, r: Box<Rule<M, ValidatorState>>) 
    {
        self.rules.push(r)
    }
}

impl <M> Validator<M> for ValidatorSchema<M> {
    fn validate(&mut self, model: &M) -> ValidationResult<bool> {
        for rule in self.rules.iter() {
            try!(rule.validate(model, &mut self.state));
        }
        
        Ok(self.state.valid)
    }
}

#[derive(Clone, Debug)]
pub struct ValidatorState
{
    pub valid: bool,
    pub errors: HashMap<String, Field>,
}

impl ValidatorState {
    pub fn new() -> ValidatorState {
        ValidatorState {
            valid: true,
            errors: HashMap::new(),
        }
    }
}

impl State for ValidatorState {
    fn accept(&mut self, field_name: &str) {
        let mut field = self.errors.entry(field_name.to_owned()).or_insert(Field::new(field_name.to_owned(), true));
        field.valid = true;
    }
    
    fn reject(&mut self, field_name: &str, reason: String) {
        let mut field = self.errors.entry(field_name.to_owned()).or_insert(Field::new(field_name.to_owned(), false));
        field.valid = false;
        field.errors.push(reason);
    }
}

