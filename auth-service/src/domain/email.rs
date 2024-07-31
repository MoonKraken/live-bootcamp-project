use std::hash::Hash; // New!

use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, Secret}; // New!
use validator::ValidateEmail;

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
    pub fn parse(s: String) -> Result<Email> {
        // ultra simple validation
        if s.validate_email() {
            Ok(Self(Secret::new(s)))
        } else {
            Err(eyre!(format!(
                "{} is not a valid email.",
                s
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
        let res: Result<Email> = Email::parse("hello".to_string());
        let expected_error_string = "hello is not a valid email.";

        // why doesn't eyre ErrReport implement PartialEq??
        match res {
            Err(e) => assert_eq!(e.to_string(), expected_error_string),
            _ => panic!("error was expected when parsing invalid email"),
        }
    }

    #[test]
    fn email_with_two_at_cannot_be_parsed() {
        let res: Result<Email> = Email::parse("hello@hello@hello.com".to_string());
        let expected_error_string = "hello@hello@hello.com is not a valid email.";

        // why doesn't eyre ErrReport implement PartialEq??
        match res {
            Err(e) => assert_eq!(e.to_string(), expected_error_string),
            _ => panic!("error was expected when parsing invalid email"),
        }
    }

    #[test]
    fn valid_email_can_be_parsed() {
        let res = Email::parse("ken@cttm.io".to_string()).expect("email should be parsed");
        let expected = "ken@cttm.io";
        //unsure whether this is a good test
        assert_eq!(res.0.expose_secret(), expected);
    }

    #[test]
    fn can_convert_email_to_string() {
        let email ="ken@cttm.io";
        let res = Email::parse(email.to_string()).unwrap();
        // not sure if this is valuable either
        assert_eq!(res.as_ref().expose_secret(), email);
    }
}
