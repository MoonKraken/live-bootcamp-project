#[derive(Hash, Debug, PartialEq, Eq, Clone)]
pub struct Password(pub String);

#[derive(Debug, PartialEq, Eq)]
pub struct PasswordError(String);

impl Password {
    pub fn parse(password: String) -> Result<Self, PasswordError> {
        if password.len() >= 8 {
            Ok(Password(password))
        } else {
            Err(PasswordError("Password too short".to_string()))
        }
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cannot_parse_short_password() {
        let res = Password::parse("a".to_string());
        let expected = Err(PasswordError("Password too short".to_string()));
        assert_eq!(res, expected);
    }

    #[test]
    fn can_parse_good_password() {
        let res = Password::parse("woefijwoeifjweoifjwoifja".to_string());
        let expected = Ok(Password("woefijwoeifjweoifjwoifja".to_string()));
        assert_eq!(res, expected);
    }

    #[test]
    fn can_convert_password_to_string() {
        let password_str = "anothersecurepassword".to_string();
        let password = Password::parse(password_str.clone()).unwrap();
        assert_eq!(password.as_ref(), password_str.as_str());
    }
}
