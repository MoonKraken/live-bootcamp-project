use color_eyre::eyre::Report;
use secrecy::{ExposeSecret, Secret};
use thiserror::Error;
use async_trait::async_trait;
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::email::Email;

// This trait represents the interface all concrete 2FA code stores should implement
#[async_trait]
pub trait TwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}

#[derive(Debug, Error)]
pub enum TwoFACodeStoreError {
    #[error("Login Attempt ID not found")]
    LoginAttemptIdNotFound,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

// New!
impl PartialEq for TwoFACodeStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::LoginAttemptIdNotFound, Self::LoginAttemptIdNotFound)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct LoginAttemptId(pub Secret<String>);

// TODO is it bad to do this?
impl PartialEq for LoginAttemptId {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl LoginAttemptId {
    pub fn parse(id: String) -> Result<Self, String> {
        // Use the `parse_str` function from the `uuid` crate to ensure `id` is a valid UUID
        let uuid = Uuid::parse_str(&id);
        match uuid {
            Ok(uuid) => Ok(Self(Secret::new(uuid.to_string()))),
            Err(_) => Err("could not parse login attempt id".to_string())
        }
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        // Use the `uuid` crate to generate a random version 4 UUID
        Self(Secret::new(Uuid::new_v4().to_string()))
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0.expose_secret()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct TwoFACode(pub Secret<String>);

impl PartialEq for TwoFACode {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl TwoFACode {
    pub fn parse(code: String) -> Result<Self, String> {
        // Ensure `code` is a valid 6-digit code
        if code.len() != 6 {
            return Err("could not parse FA Code".to_string());
        }
        if !code.chars().all(|c| c.is_digit(10)) {
            return Err("could not parse FA Code because a non-digit was found".to_string());
        }

        Ok(Self(Secret::new(code)))
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        // Use the `rand` crate to generate a random 2FA code.
        // The code should be 6 digits (ex: 834629)
        let mut rng = rand::thread_rng();
        let code: u32 = rng.gen_range(100000..1000000);
        Self(Secret::new(code.to_string()))
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0.expose_secret()
    }
}
