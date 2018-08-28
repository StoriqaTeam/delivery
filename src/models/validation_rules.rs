use std::borrow::Cow;
use std::collections::HashMap;

use serde_json;
use stq_static_resources::Translation;
use validator::ValidationError;

pub fn validate_non_negative<T: Into<f64>>(val: T) -> Result<(), ValidationError> {
    if val.into() > 0f64 {
        Ok(())
    } else {
        Err(ValidationError {
            code: Cow::from("value"),
            message: Some(Cow::from("Value must be non negative.")),
            params: HashMap::new(),
        })
    }
}

pub fn validate_translation(text: &serde_json::Value) -> Result<(), ValidationError> {
    let translations = serde_json::from_value::<Vec<Translation>>(text.clone()).map_err(|_| ValidationError {
        code: Cow::from("text"),
        message: Some(Cow::from("Invalid json format of text with translation.")),
        params: HashMap::new(),
    })?;

    for t in translations {
        if t.text.is_empty() {
            return Err(ValidationError {
                code: Cow::from("text"),
                message: Some(Cow::from("Text inside translation must not be empty.")),
                params: HashMap::new(),
            });
        }
    }

    Ok(())
}

pub fn validate_urls(text: &serde_json::Value) -> Result<(), ValidationError> {
    serde_json::from_value::<Vec<String>>(text.clone()).map_err(|_| ValidationError {
        code: Cow::from("urls"),
        message: Some(Cow::from("Invalid format of urls. Must be json array of strings.")),
        params: HashMap::new(),
    })?;

    Ok(())
}
