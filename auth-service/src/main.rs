use auth_service::app_state::AppState;
use auth_service::app_state::BannedTokenStoreType;
use auth_service::app_state::EmailClientType;
use auth_service::app_state::TwoFACodeStoreType;
use auth_service::app_state::UserStoreType;
use auth_service::get_postgres_pool;
use auth_service::services::data_stores::hashmap_two_fa_code_store::HashmapTwoFACodeStore;
use auth_service::services::data_stores::hashset_banned_token_store::HashsetBannedTokenStore;
use auth_service::services::data_stores::postgres_user_store::PostgresUserStore;
use auth_service::services::mock_email_client::MockEmailClient;
use auth_service::utils::constants::prod;
use auth_service::utils::constants::DATABASE_URL;
use auth_service::Application;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    // We will use this PostgreSQL pool in the next task!
    let pg_pool = configure_postgresql().await;
    let user_store: UserStoreType = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
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

async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database!
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}
