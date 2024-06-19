use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, Secret};

#[derive(Debug, Clone)]
pub struct Password(pub Secret<String>);

#[derive(Debug, PartialEq, Eq)]
pub struct PasswordError(String);

impl PartialEq for Password {
    // New!
    fn eq(&self, other: &Self) -> bool {
        // We can use the expose_secret method to expose the secret in a
        // controlled manner when needed!
        self.0.expose_secret() == other.0.expose_secret() // Updated!
    }
}

impl Password {
    pub fn parse(s: Secret<String>) -> Result<Password> {
        // Updated!
        if validate_password(&s) {
            Ok(Self(s))
        } else {
            Err(eyre!("Failed to parse string to a Password type"))
        }
    }
}

fn validate_password(s: &Secret<String>) -> bool {
    // Updated!
    s.expose_secret().len() >= 8
}

impl AsRef<Secret<String>> for Password {
    // Updated!
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Password;

    use fake::faker::internet::en::Password as FakePassword;
    use fake::Fake;
    use secrecy::Secret;

    #[test]
    fn cannot_parse_short_password() {
        let secret = Secret::new("a".to_string());
        assert!(Password::parse(secret).is_err());
    }

    #[derive(Debug, Clone)]
    struct ValidPasswordFixture(pub Secret<String>); // Updated!

    // TODO this deviates from Bogdan's example code, eventually maybe figure out how to use g
    // for the random generation
    impl quickcheck::Arbitrary for ValidPasswordFixture {
        fn arbitrary(_g: &mut quickcheck::Gen) -> Self {
            let mut rng = rand::thread_rng();
            let password: String = FakePassword(8..30).fake_with_rng(&mut rng);
            Self(Secret::new(password))
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_passwords_are_parsed_successfully(valid_password: ValidPasswordFixture) -> bool {
        Password::parse(valid_password.0).is_ok()
    }
}
