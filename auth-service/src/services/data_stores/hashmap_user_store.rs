use std::collections::HashMap;

use crate::domain::email::Email;
use crate::domain::password::Password;
use crate::domain::user::User;
use crate::domain::data_stores::{UserStore, UserStoreError};

// TODO: Create a new struct called `HashmapUserStore` containing a `users` field
// which stores a `HashMap`` of email `String`s mapped to `User` objects.
// Derive the `Default` trait for `HashmapUserStore`.
#[derive(Default)]
pub struct HashMapUserStore {
    users: HashMap<Email, User>,
}

#[async_trait::async_trait]
impl UserStore for HashMapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        // Return `UserStoreError::UserAlreadyExists` if the user already exists,
        // otherwise insert the user into the hashmap and return `Ok(())`.
        if self.users.contains_key(&user.email) {
            Err(UserStoreError::UserAlreadyExists)
        } else {
            self.users.insert(user.email.clone(), user);
            Ok(())
        }
    }

    // TODO: Implement a public method called `get_user`, which takes an
    // immutable reference to self and an email string slice as arguments.
    // This function should return a `Result` type containing either a
    // `User` object or a `UserStoreError`.
    // Return `UserStoreError::UserNotFound` if the user can not be found.

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok(user.clone()),
            _ => Err(UserStoreError::UserNotFound),
        }
    }
    // TODO: Implement a public method called `validate_user`, which takes an
    // immutable reference to self, an email string slice, and a password string slice
    // as arguments. `validate_user` should return a `Result` type containing either a
    // unit type `()` if the email/password passed in match an existing user, or a `UserStoreError`.
    // Return `UserStoreError::UserNotFound` if the user can not be found.
    // Return `UserStoreError::InvalidCredentials` if the password is incorrect.

    async fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            Some(user) => {
                if &user.password == password {
                    Ok(())
                } else {
                    Err(UserStoreError::InvalidCredentials)
                }},
            _ => Err(UserStoreError::UserNotFound)
        }
    }
}

// TODO: Add unit tests for your `HashmapUserStore` implementation
#[cfg(test)]
mod tests {
    use secrecy::Secret;

    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut test_store = HashMapUserStore::default();

        let email = Email::parse("email@yahoo.net".to_string()).unwrap();
        let secret = Secret::new("passwordistaco".to_string());
        let test_user = User {
            email,
            password: Password::parse(secret).unwrap(),
            requires_2fa: false,
        };

        let res = test_store.add_user(test_user).await;

        assert_eq!(res, Ok(()));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut test_store = HashMapUserStore::default();
        let email = Email::parse("email@yahoo.net".to_string()).unwrap();
        let secret = Secret::new("passwordistaco".to_string());
        let test_user = User {
            email: email.clone(),
            password: Password::parse(secret).unwrap(),
            requires_2fa: false,
        };

        let _ = test_store.add_user(test_user.clone()).await;

        let get_res = test_store.get_user(&email).await;

        assert_eq!(get_res, Ok(test_user));
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut test_store = HashMapUserStore::default();
        let email = Email::parse("email@yahoo.net".to_string()).unwrap();
        let secret = Secret::new("passwordistaco".to_string());
        let test_user = User {
            email,
            password: Password::parse(secret).unwrap(),
            requires_2fa: false,
        };

        let _ = test_store.add_user(test_user.clone()).await;

        let validate_res = test_store.validate_user(&test_user.email, &test_user.password).await;

        assert_eq!(validate_res, Ok(()));
    }
}
