use std::sync::Arc;
use std::sync::RwLock;

use crate::services::hashmap_user_store::HashMapUserStore;

// Using a type alias to improve readability!
pub type UserStoreType = Arc<RwLock<HashMapUserStore>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
}

impl AppState {
    pub fn new(user_store: UserStoreType) -> Self {
        Self { user_store }
    }
}
