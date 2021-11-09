use crate::{Error, Validator, ValidatorBase, Value};
use std::collections::HashMap;

pub fn object(key_validators: HashMap<String, Box<dyn Validator>>) -> Box<dyn Validator> {
    Box::new(ObjectValidator {
        key_validators,
        strict: false,
    })
}

struct ObjectValidator {
    key_validators: HashMap<String, Box<dyn Validator>>,
    strict: bool,
}

impl ValidatorBase for ObjectValidator {
    fn validate<'a>(&self, value: &'a Value) -> Result<(), Error<'a>> {
        let object = value
            .as_object()
            .ok_or(Error::InvalidType(value, String::from("object")))?;

        for (key, validator) in &self.key_validators {
            let inner_value = object
                .get(key)
                .ok_or(Error::MissingObjectKey(value, key.clone()))?;

            validator.validate(inner_value)?
        }

        if self.strict {
            // Make sure there is no other keys than the one defined in the validator
            // if we are in strict mode.
            for (key, value) in object {
                self.key_validators
                    .get(key)
                    .ok_or(Error::UnexpectedObjectKey(value, key.clone()))
                    .map(|_| ())?
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::validators;
    use crate::{Error, Validator};

    #[test]
    fn valid() {
        let mut key_validators: HashMap<String, Box<dyn Validator>> = HashMap::new();
        key_validators.insert(String::from("key"), validators::string(|_| Ok(())));

        let validator = super::object(key_validators);
        assert_eq!(
            Ok(()),
            validator.validate(&serde_json::json!({"key": "val"}))
        );
    }

    #[test]
    fn missing_key() {
        let mut key_validators: HashMap<String, Box<dyn Validator>> = HashMap::new();
        key_validators.insert(String::from("key"), validators::string(|_| Ok(())));

        let validator = super::object(key_validators);
        assert!(matches!(
            validator.validate(&serde_json::json!({})),
            Err(Error::MissingObjectKey(_, _))
        ));
    }
}
