#[derive(Hash, Debug, PartialEq, Eq, Clone)]
pub struct Email(String);

#[derive(Debug, PartialEq, Eq)]
pub struct EmailError(String);

impl Email {
    pub fn parse(email: String) -> Result<Self, EmailError> {
        if email.contains("@") {
            Ok(Email(email))
        } else {
            Err(EmailError("Could not parse email address".to_string()))
        }
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_without_at_cannot_be_parsed() {
        let res = Email::parse("hello".to_string());
        let expected_error = Err(EmailError("Could not parse email address".to_string()));
        assert_eq!(res, expected_error)
    }

    #[test]
    fn valid_email_can_be_parsed() {
        let res = Email::parse("ken@cttm.io".to_string());
        let expected = Ok(Email("ken@cttm.io".to_string()));
        assert_eq!(res, expected)
    }

    #[test]
    fn can_convert_email_to_string() {
        let email_str = "ken@cttm.io".to_string();
        let email = Email::parse(email_str.clone()).unwrap();
        assert_eq!(email.as_ref(), email_str.as_str());
    }
}
