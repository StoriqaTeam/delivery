#[macro_export]

/// Macro for creating validation errors. It uses json-like syntax
/// `{<field_name>: [<error_code> => <error_message>]`. Field is coming
/// from struct like
///
/// ```
///   struct Form {
///     email: String,
///     password: String
///   }
/// ```
/// In this case email and password are `field_names`.
///
/// `error_code` is smth like "too long", "not an email", etc -
/// i.e. the type of validator that fails. Always
/// use `validator::Validator` enum for that, unless it really doesn't fit.
/// `error_message` is a custom error message.
///
/// # Examples
///
/// ```
/// #[macro_use] extern crate delivery_lib;
/// extern crate validator;
///
/// use validator::Validator;
///
/// fn main() {
///     let errors = validation_errors!({
///         "email": [Validator::Email.code() => "Invalid email", "exists" => "Already exists"],
///         "password": ["match" => "Doesn't match"]
///     });
/// }
/// ```

macro_rules! validation_errors {
    ({$($field:tt: [$($code:expr => $value:expr),+]),*}) => {{
        use validator;
        use std::borrow::Cow;
        use std::collections::HashMap;

        let mut errors = validator::ValidationErrors::new();
        $(
            $(
                let error = validator::ValidationError {
                    code: Cow::from($code),
                    message: Some(Cow::from($value)),
                    params: HashMap::new(),
                };

                errors.add($field, error);
            )+
        )*

        errors
    }}
}

#[cfg(test)]
mod tests {
    use serde_json;
    use validator::Validator;

    #[test]
    fn several_errors() {
        let errors = validation_errors!({
            "email": [Validator::Email.code() => "Invalid email", "exists" => "Already exists"],
            "password": ["match" => "Doesn't match"]
        });
        let json = serde_json::from_str::<serde_json::Value>(&serde_json::to_string(&errors).unwrap()).unwrap();

        assert_eq!(json["email"][0]["code"], "email");
        assert_eq!(json["email"][0]["message"], "Invalid email");
        assert_eq!(json["email"][1]["code"], "exists");
        assert_eq!(json["email"][1]["message"], "Already exists");
        assert_eq!(json["password"][0]["code"], "match");
        assert_eq!(json["password"][0]["message"], "Doesn't match");
    }
}
