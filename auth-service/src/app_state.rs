use std::sync::Arc;
use std::sync::RwLock;

use crate::domain::data_stores::banned_token_store::BannedTokenStore;
use crate::domain::data_stores::UserStore;

// Using a type alias to improve readability!
pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;
pub type BannedTokenStoreType = Arc<RwLock<dyn BannedTokenStore + Send + Sync>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenStoreType,
}

impl AppState {
    pub fn new(user_store: UserStoreType, banned_token_store: BannedTokenStoreType) -> Self {
        Self { user_store, banned_token_store }
    }
}
