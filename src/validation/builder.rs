use super::state::*;
use super::result;

pub trait BuilderRule<T, A, S>
{
	fn build(&self, input:&T, accumulator: &mut A, state: &mut S) -> result::Result<()>;
}

impl <T, A, S, F> BuilderRule<T, A, S> for F
where F: Fn(&T, &mut A, &mut S) -> result::Result<()>
{
    fn build(&self, input:&T, accumulator: &mut A, state: &mut S) -> result::Result<()> {
        (*self)(input, accumulator, state)
    }
}

pub struct Builder<M, A> {
    pub state: ValidationState,
    pub rules: Vec<Box<BuilderRule<M, A, ValidationState>>>
}

impl <M, A> Builder<M, A> {
    pub fn new() -> Self {
        Builder {
            state: ValidationState::new(),
            rules: vec![],
        }
    }
    
    pub fn rule(&mut self, r: Box<BuilderRule<M, A, ValidationState>>) 
    {
        self.rules.push(r)
    }
    
    pub fn build<'a>(&mut self, model: &M, accumulator: &'a mut A) -> result::Result<&'a A> {
        for rule in self.rules.iter() {
            if let Err(err) = rule.build(model, accumulator, &mut self.state) {
                self.state.valid = false;
                self.state.errors.push(err);
            }
        }
        
        Ok(accumulator)
    }
    
    pub fn is_valid(&self) -> bool {
        self.state.valid
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    
    use super::*;
    use super::super::result;
    use super::super::params::*;
    use super::super::state::*;
    
    #[derive(Clone, Debug, PartialEq)]
    pub struct TestStruct {
        number: i32,
        maybe_number: Option<i32>,
        text: String,
        maybe_text: Option<String>
    }
    
    #[derive(Clone, Debug)]
    pub struct TestStructBuilder {
        number: Option<i32>,
        maybe_number: Option<i32>,
        text: Option<String>,
        maybe_text: Option<String>,
        
        state: ValidationState,
    }
    
    impl TestStructBuilder {
        pub fn build(self) -> result::Result<TestStruct> {
            if self.state.valid {
                Ok(TestStruct {
                    number: try!(self.number.ok_or(result::ValidationError::MissingRequiredValue("number".to_owned()))),
                    maybe_number: self.maybe_number,
                    text: try!(self.text.ok_or(result::ValidationError::MissingRequiredValue("text".to_owned()))),
                    maybe_text: self.maybe_text
                })
            } else {
                Err(result::ValidationError::ValidationError(self.state))
            }
        }
        
        pub fn load_values(&mut self, values: &HashMap<String, Vec<String>>) -> result::Result<bool> {
            if let Some(number) = try!(multimap_get_maybe_one(values, "number")) {
                if let Ok(number) = number.parse::<i32>() {
                    self.number = Some(number);
                } else {
                    self.state.reject("number", result::ValidationError::InvalidValue("test msg".to_owned()));
                }
            } else {
                self.state.reject("number", result::ValidationError::MissingRequiredValue("test msg".to_owned()));
            }
            
            if let Some(text) = try!(multimap_get_maybe_one(values, "text")) {
                self.text = Some(text.to_owned());
            } else {
                self.state.reject("text", result::ValidationError::MissingRequiredValue("test msg".to_owned()));
            }
            
            Ok(self.state.valid)
        }
        
        pub fn from_hashmap(values: &HashMap<String, Vec<String>>) -> result::Result<TestStruct> {
            let mut accumulator = TestStructBuilder {
                number: None,
                maybe_number: None,
                text: None,
                maybe_text: None,
                
                state: ValidationState::new(),
            };
            
            try!(accumulator.load_values(values));
            
            accumulator.build()
        }
    }
    
    #[test]
    fn test_null_builder() {
        let mut accumulator = TestStructBuilder {
            number: None,
            maybe_number: None,
            text: None,
            maybe_text: None,
            
            state: ValidationState::new(),
        };
            
        let values = HashMap::new();
        let mut builder = Builder::<HashMap<String,Vec<String>>, TestStructBuilder>::new();
        
        assert_eq!(builder.build(&values, &mut accumulator).is_ok(), true);
    }
    
    #[test]
    fn test_simple_builder() {
        let mut values = HashMap::new();
        values.insert("number".to_owned(), vec!["123".to_owned()]);
        values.insert("text".to_owned(), vec!["some text".to_owned()]);

        assert_eq!(TestStructBuilder::from_hashmap(&values).is_ok(), true);
    }
    
    #[test]
    fn test_failed_builder() {
        let mut values = HashMap::new();
        values.insert("number".to_owned(), vec!["123".to_owned()]);

        let build_result = TestStructBuilder::from_hashmap(&values);
        
        assert_eq!(build_result.is_ok(), false);
        
        if let result::ValidationError::ValidationError(state) = build_result.err().unwrap() {
            assert_eq!(state.errors.is_empty(), true);
            assert_eq!(state.fields.get("text").unwrap().errors.len(), 1);
            assert_eq!(format!("{}", state.fields.get("text").unwrap().errors.get(0).unwrap()), "missing required value: test msg");
        } else {
            assert!(false);
        }
    }
    
    #[test]
    fn test_err_builder() {
        let mut accumulator = TestStructBuilder {
            number: None,
            maybe_number: None,
            text: None,
            maybe_text: None,
            
            state: ValidationState::new(),
        };
            
        let mut builder = Builder::<String, TestStructBuilder>::new();
            
        builder.rule(Box::new(|_input: &String, _accumulator: &mut TestStructBuilder, _state: &mut ValidationState| -> result::Result<()> {
            Err(result::ValidationError::ApplicationError("test error".to_owned()))
        }));
        
        assert_eq!(builder.build(&String::new(), &mut accumulator).is_ok(), true);
        assert_eq!(builder.state.valid, false);
        assert_eq!(builder.state.errors.len(), 1);
        assert_eq!(format!("{}", builder.state.errors.get(0).unwrap()), "Application error: test error");
    }
}
