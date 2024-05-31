use std::collections::HashSet;

use crate::domain::data_stores::banned_token_store::BannedTokenStore;

#[derive(Default)]
pub struct HashsetBannedTokenStore {
    store: HashSet<String>,
}

impl BannedTokenStore for HashsetBannedTokenStore {
    fn add_token(&mut self, token: String) -> () {
        self.store.insert(token);
    }

    fn contains_token(&self, token: &str) -> bool {
        if self.store.contains(token) {
            true
        } else {
            false
        }
    }
}
