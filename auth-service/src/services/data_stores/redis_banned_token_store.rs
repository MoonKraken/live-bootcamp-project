use std::sync::Arc;

use redis::{Commands, Connection};
use secrecy::{ExposeSecret, Secret};
use tokio::sync::RwLock;

use crate::{
    domain::data_stores::banned_token_store::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[tracing::instrument(name = "Add Token", skip_all)]
    async fn add_token(&mut self, token: Secret<String>) -> Result<(), BannedTokenStoreError> {
        let key = get_key(&token.expose_secret());
        let mut write_lock = self.conn.write().await;
        write_lock
            .set_ex(key, true, TOKEN_TTL_SECONDS as u64)
            .map_err(|e| BannedTokenStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    #[tracing::instrument(name = "Contains Token", skip_all)]
    async fn contains_token(&self, token: &Secret<String>) -> Result<bool, BannedTokenStoreError> {
        // Check if the token exists by calling the exists method on the Redis connection
        let key = get_key(&token.expose_secret());
        // TODO tried getting a read lock, didn't work for some reason
        let mut read_lock = self.conn.write().await;

        let exists = read_lock
            .exists(key)
            .map_err(|e| BannedTokenStoreError::UnexpectedError(e.into()))?;

        Ok(exists)
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
