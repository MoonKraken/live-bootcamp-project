use auth_service::app_state::AppState;
use auth_service::app_state::BannedTokenStoreType;
use auth_service::app_state::TwoFACodeStoreType;
use auth_service::app_state::UserStoreType;
use auth_service::domain::Email;
use auth_service::get_postgres_pool;
use auth_service::get_redis_client;
use auth_service::services::data_stores::postgres_user_store::PostgresUserStore;
use auth_service::services::data_stores::redis_banned_token_store::RedisBannedTokenStore;
use auth_service::services::data_stores::redis_two_fa_code_store::RedisTwoFACodeStore;
use auth_service::services::postmark_email_client::PostmarkEmailClient;
use auth_service::utils::constants::prod;
use auth_service::utils::constants::DATABASE_URL;
use auth_service::utils::constants::POSTMARK_AUTH_TOKEN;
use auth_service::utils::constants::REDIS_HOST_NAME;
use auth_service::utils::tracing::init_tracing;
use auth_service::Application;
use reqwest::Client;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to initialize tracing");
    // We will use this PostgreSQL pool in the next task!
    let pg_pool = configure_postgresql().await;
    let user_store: UserStoreType = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let redis_connection = Arc::new(RwLock::new(configure_redis()));
    let banned_token_store: BannedTokenStoreType = Arc::new(RwLock::new(
        RedisBannedTokenStore::new(redis_connection.clone()),
    ));
    let two_fa_store: TwoFACodeStoreType =
        Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_connection)));
    let email_client = Arc::new(RwLock::new(configure_postmark_email_client()));
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

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

fn configure_postmark_email_client() -> PostmarkEmailClient {
    let http_client = Client::builder()
        .timeout(prod::email_client::TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    PostmarkEmailClient::new(
        prod::email_client::BASE_URL.to_owned(),
        Email::parse(prod::email_client::SENDER.to_owned()).unwrap(),
        POSTMARK_AUTH_TOKEN.to_owned(),
        http_client,
    )
}
