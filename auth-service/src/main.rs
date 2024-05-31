use auth_service::app_state::AppState;
use auth_service::app_state::BannedTokenStoreType;
use auth_service::app_state::UserStoreType;
use auth_service::services::hashmap_user_store::HashMapUserStore;
use auth_service::services::hashset_banned_token_store::HashsetBannedTokenStore;
use auth_service::utils::constants::prod;
use auth_service::Application;
use std::sync::Arc;
use std::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store: UserStoreType = Arc::new(RwLock::new(HashMapUserStore::default()));
    let banned_token_store: BannedTokenStoreType =
        Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
    let app_state = AppState::new(user_store, banned_token_store);
    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
