pub mod banned_token_store;
pub mod two_fa_code_store;
pub use two_fa_code_store::*;
use super::{email::Email, password::Password, user::User};

#[async_trait::async_trait]
pub trait UserStore {
    fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}
