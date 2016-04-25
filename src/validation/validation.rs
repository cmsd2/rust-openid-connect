use super::state::*;
use super::result::{Result};

pub trait Validator<M> {
    fn validate(&mut self, model: &M) -> Result<bool>;
}

pub trait Rule<T, S>
{
	fn validate(&self, input:&T, state: &mut S) -> Result<()>;
}

impl <T, S, F> Rule<T, S> for F
where F: Fn(&T, &mut S) -> Result<()>
{
    fn validate(&self, input:&T, state: &mut S) -> Result<()> {
        (*self)(input, state)
    }
}

pub struct ValidationSchema<M> {
    pub state: ValidationState,
    pub rules: Vec<Box<Rule<M, ValidationState>>>
}

impl <M> ValidationSchema<M> {
    pub fn new() -> Self {
        ValidationSchema {
            state: ValidationState::new(),
            rules: vec![],
        }
    }
    
    pub fn rule(&mut self, r: Box<Rule<M, ValidationState>>) 
    {
        self.rules.push(r)
    }
}

impl <M> Validator<M> for ValidationSchema<M> {
    fn validate(&mut self, model: &M) -> Result<bool> {
        for rule in self.rules.iter() {
            if let Err(err) = rule.validate(model, &mut self.state) {
                self.state.valid = false;
                self.state.errors.push(err);
            }
        }
        
        Ok(self.state.valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::result::*;
    use super::super::state::*;
    
    #[allow(dead_code)]
    struct TestStruct {
        num: i32,
        text: String,
    }
    
    impl TestStruct {
        pub fn new<T>(num: i32, text: T) -> TestStruct where T: Into<String> {
            TestStruct {
                num: num,
                text: text.into(),
            }
        }
    }
    
    #[test]
    pub fn test_null_rule() {
        let mut v = ValidationSchema::<TestStruct>::new();
        
        v.rule(Box::new(|_m: &TestStruct, _vs: &mut ValidationState| {
            Ok(())
        }));
        
        let a = TestStruct::new(123, "hello");
        
        assert_eq!(v.validate(&a).unwrap_or(false), true);
    }
    
    #[test]
    pub fn test_accept_rule() {
        let mut v = ValidationSchema::<TestStruct>::new();
        
        v.rule(Box::new(|_m: &TestStruct, vs: &mut ValidationState| {
            vs.accept("field name");
            Ok(())
        }));
        
        let a = TestStruct::new(123, "hello");
        
        assert_eq!(v.validate(&a).unwrap_or(false), true);
    }
    
    #[test]
    pub fn test_reject_rule() {
        let mut v = ValidationSchema::<TestStruct>::new();
        
        v.rule(Box::new(|_m: &TestStruct, vs: &mut ValidationState| {
            vs.reject("field name", ValidationError::InvalidValue("test error".to_owned()));
            Ok(())
        }));
        
        let a = TestStruct::new(123, "hello");
        
        assert_eq!(v.validate(&a).unwrap_or(true), false);
    }
    
    #[test]
    pub fn test_err_rule() {
        let mut v = ValidationSchema::<TestStruct>::new();
        
        v.rule(Box::new(|_m: &TestStruct, _vs: &mut ValidationState| -> Result<()> {
            Err(ValidationError::ApplicationError("test error".to_owned()))
        }));
        
        let a = TestStruct::new(123, "hello");
        
        assert_eq!(v.validate(&a).unwrap_or(true), false);
        assert_eq!(v.state.errors.len(), 1);
        assert_eq!(format!("{}", v.state.errors.get(0).unwrap()), "Application error: test error");
    }
}