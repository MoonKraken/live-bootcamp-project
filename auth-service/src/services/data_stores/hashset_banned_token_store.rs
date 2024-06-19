use std::collections::HashSet;

use secrecy::{ExposeSecret, Secret};

use crate::domain::data_stores::banned_token_store::{BannedTokenStore, BannedTokenStoreError};

#[derive(Default)]
pub struct HashsetBannedTokenStore {
    store: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_token(&mut self, token: Secret<String>) -> Result<(), BannedTokenStoreError> {
        self.store.insert(token.expose_secret().to_string());
        Ok(())
    }

    async fn contains_token(&self, token: &Secret<String>) -> Result<bool, BannedTokenStoreError> {
        if self.store.contains(token.expose_secret()) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
