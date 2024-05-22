use std::sync::Arc;
use std::sync::RwLock;
use auth_service::app_state::UserStoreType;
use auth_service::services::hashmap_user_store::HashMapUserStore;
use auth_service::app_state::AppState;
use auth_service::Application;

#[tokio::main]
async fn main() {
    let user_store: UserStoreType = Arc::new(RwLock::new(HashMapUserStore::default()));
    let app_state = AppState::new(user_store);
    let app = Application::build(app_state, "0.0.0.0:3000")
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
