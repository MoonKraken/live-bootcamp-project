use color_eyre::eyre::{Context, Result};

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
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let _ = sqlx::query("INSERT INTO users VALUES ($1, $2, $3)")
            .bind(user.email.as_ref())
            .bind(
                compute_password_hash(user.password.as_ref().to_string()).await
                    .map_err(|e| UserStoreError::UnexpectedError(e))?,
            )
            .bind(user.requires_2fa)
            .execute(&self.pool)
            .await
            .map_err(|_| {
                // technically we should be looking at e to make sure the failure
                // really was due to a primary key conflict, but it seemed
                // like doing that was going to be nontrivial, so just
                // assume for now
                UserStoreError::UserAlreadyExists
            })?;

        Ok(())
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
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
                    _ => UserStoreError::UnexpectedError(e.into()),
                }
            })?;

        let email: String = res
            .try_get("email")
            .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;
        let password: String = res
            .try_get("password_hash")
            .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        Ok(User {
            email: Email(email),
            password: Password(password),
            requires_2fa: res
                .try_get("requires_2fa")
                .map_err(|e| UserStoreError::UnexpectedError(e.into()))?,
        })
    }

    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
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
                dbg!(&e);
                match e {
                    sqlx::Error::RowNotFound => UserStoreError::UserNotFound,
                    _ => UserStoreError::UnexpectedError(e.into()),
                }
            })?;

        let expected_hash: String = res
            .try_get("password_hash")
            .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        let password = password.clone();
        match verify_password_hash(expected_hash, password.as_ref().to_string()).await {
            Ok(()) => Ok(()),
            _ => Err(UserStoreError::InvalidCredentials),
        }
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))
    }
}

// Helper function to verify if a given password matches an expected hash
// TODO: Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking
#[tracing::instrument(name = "Verify password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<()> {
    // This line retrieves the current span from the tracing context.
    // The span represents the execution context for the compute_password_hash function.
    let current_span: tracing::Span = tracing::Span::current(); // New!
    let result = tokio::task::spawn_blocking(move || {
        // This code block ensures that the operations within the closure are executed within the context of the current span.
        // This is especially useful for tracing operations that are performed in a different thread or task, such as within tokio::task::spawn_blocking.
        current_span.in_scope(|| {
            // New!
            let expected_password_hash: PasswordHash<'_> =
                PasswordHash::new(&expected_password_hash)?;

            Argon2::default()
                .verify_password(password_candidate.as_bytes(), &expected_password_hash)
                .wrap_err("failed to verify password hash")
        })
    })
    .await;

    result?
}

// Helper function to hash passwords before persisting them in the database.
// TODO: Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking
#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: String) -> Result<String> {
    // This line retrieves the current span from the tracing context.
    // The span represents the execution context for the compute_password_hash function.
    let current_span: tracing::Span = tracing::Span::current(); // New!

    let result = tokio::task::spawn_blocking(move || {
        // This code block ensures that the operations within the closure are executed within the context of the current span.
        // This is especially useful for tracing operations that are performed in a different thread or task, such as within tokio::task::spawn_blocking.
        current_span.in_scope(|| {
            let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

            Ok(password_hash)
        })
    })
    .await;

    result?
}
