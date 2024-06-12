use std::error::Error;

use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};

use sqlx::{PgPool, Row};

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    Email, Password, User,
};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let res = sqlx::query("INSERT INTO users VALUES ($1, $2, $3)")
            .bind(user.email.as_ref())
            .bind(
                compute_password_hash(user.password.as_ref())
                    .map_err(|_| UserStoreError::UnexpectedError)?,
            )
            .bind(user.requires_2fa)
            .execute(&self.pool)
            .await
            .map_err(|_| {
                // assume for now that it's because the user already exists
                // there doesn't seem to be a specific error type for conflicting primary key
                UserStoreError::UserAlreadyExists
            })?;

        Ok(())
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let res = sqlx::query("SELECT * FROM users WHERE email = $1")
            .bind(email.as_ref())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                println!("error getting user for validation");
                dbg!(&e);
                match e {
                    sqlx::Error::RowNotFound => UserStoreError::UserNotFound,
                    _ => UserStoreError::UnexpectedError,
                }
            })?;

        let email: String = res
            .try_get("email")
            .map_err(|_| UserStoreError::UnexpectedError)?;
        let password: String = res
            .try_get("password_hash")
            .map_err(|_| UserStoreError::UnexpectedError)?;

        Ok(User {
            email: Email(email),
            password: Password(password),
            requires_2fa: res
                .try_get("requires_2fa")
                .map_err(|_| UserStoreError::UnexpectedError)?,
        })
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let res = sqlx::query("SELECT * FROM users WHERE email = $1")
            .bind(email.as_ref())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                let email = email.as_ref();
                let password = password.as_ref();
                println!("error getting user for validation: {email}:{password}");
                dbg!(&e);
                match e {
                    sqlx::Error::RowNotFound => UserStoreError::UserNotFound,
                    _ => UserStoreError::UnexpectedError,
                }
            })?;

        let expected_hash: String = res
            .try_get("password_hash")
            .map_err(|_| UserStoreError::UnexpectedError)?;

        let password = password.clone();
        tokio::task::spawn_blocking(move || {
            match verify_password_hash(&expected_hash, password.as_ref()) {
                Ok(()) => Ok(()),
                _ => Err(UserStoreError::InvalidCredentials),
            }
        })
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?
    }
}

// Helper function to verify if a given password matches an expected hash
// TODO: Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking
fn verify_password_hash(
    expected_password_hash: &str,
    password_candidate: &str,
) -> Result<(), Box<dyn Error>> {
    let expected_password_hash: PasswordHash<'_> = PasswordHash::new(expected_password_hash)?;

    Argon2::default()
        .verify_password(password_candidate.as_bytes(), &expected_password_hash)
        .map_err(|e| e.into())
}

// Helper function to hash passwords before persisting them in the database.
// TODO: Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking
fn compute_password_hash(password: &str) -> Result<String, Box<dyn Error>> {
    let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None)?,
    )
    .hash_password(password.as_bytes(), &salt)?
    .to_string();

    Ok(password_hash)
}
