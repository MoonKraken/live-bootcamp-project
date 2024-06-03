use auth_service::app_state::AppState;
use auth_service::app_state::BannedTokenStoreType;
use auth_service::app_state::EmailClientType;
use auth_service::app_state::TwoFACodeStoreType;
use auth_service::app_state::UserStoreType;
use auth_service::services::hashmap_two_fa_code_store::HashmapTwoFACodeStore;
use auth_service::services::hashmap_user_store::HashMapUserStore;
use auth_service::services::hashset_banned_token_store::HashsetBannedTokenStore;
use auth_service::services::mock_email_client::MockEmailClient;
use auth_service::utils::constants::prod;
use auth_service::Application;
use std::sync::Arc;
use std::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store: UserStoreType = Arc::new(RwLock::new(HashMapUserStore::default()));
    let banned_token_store: BannedTokenStoreType =
        Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
    let two_fa_store: TwoFACodeStoreType = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
    let email_client: EmailClientType = Arc::new(RwLock::new(MockEmailClient::default()));
    let app_state = AppState::new(user_store, banned_token_store, two_fa_store, email_client);
    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
