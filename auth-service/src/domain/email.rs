use std::hash::Hash; // New!

use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, Secret}; // New!

#[derive(Debug, Clone)]
pub struct Email(pub Secret<String>);

// #[derive(Debug, PartialEq, Eq)]
// pub struct EmailError(String);

impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Hash for Email {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state);
    }
}

// New!
impl Eq for Email {}

impl Email {
    pub fn parse(s: Secret<String>) -> Result<Email> {
        // ultra simple validation
        if s.expose_secret().contains("@") {
            Ok(Self(s))
        } else {
            Err(eyre!(format!(
                "{} is not a valid email.",
                s.expose_secret()
            )))
        }
    }
}

// Updated!
impl AsRef<Secret<String>> for Email {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_without_at_cannot_be_parsed() {
        let secret_email = Secret::new("hello".to_string());
        let res: Result<Email> = Email::parse(secret_email);
        let expected_error_string = "hello is not a valid email.";

        // why doesn't eyre ErrReport implement PartialEq??
        match res {
            Err(e) => assert_eq!(e.to_string(), expected_error_string),
            _ => panic!("error was expected when parsing invalid email"),
        }
    }

    #[test]
    fn valid_email_can_be_parsed() {
        let secret_email = Secret::new("ken@cttm.io".to_string());
        let res = Email::parse(secret_email.clone()).expect("email should be parsed");
        let expected: Email = Email(secret_email);
        //unsure whether this is a good test
        assert_eq!(res, expected);
    }

    #[test]
    fn can_convert_email_to_string() {
        let secret_email = Secret::new("ken@cttm.io".to_string());
        let res = Email::parse(secret_email.clone()).unwrap();
        // not sure if this is valuable either
        assert_eq!(res.as_ref().expose_secret(), secret_email.expose_secret());
    }
}
