use auth_service::services::data_stores::postgres_user_store::PostgresUserStore;
use auth_service::services::data_stores::redis_banned_token_store::RedisBannedTokenStore;
use auth_service::utils::constants::{DATABASE_URL, REDIS_HOST_NAME};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

use auth_service::app_state::{
    AppState, BannedTokenStoreType, EmailClientType, TwoFACodeStoreType,
};
use auth_service::services::data_stores::hashmap_two_fa_code_store::HashmapTwoFACodeStore;
use auth_service::services::mock_email_client::MockEmailClient;
use auth_service::utils::constants::test::APP_ADDRESS;
use auth_service::{get_postgres_pool, get_redis_client, Application};
use reqwest::cookie::Jar;
use uuid::Uuid;
pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_store: TwoFACodeStoreType,
    pub db_name: String,
    pub clean_up_called: bool,
}

// ideally we'd call cleanup here but Bogdan says it's not possible because Async destructors are not supported
impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.clean_up_called {
            panic!("cleanup wasn't called!")
        }
    }
}

impl TestApp {
    pub async fn new() -> Self {
        let db_name = Uuid::new_v4().to_string();
        let pg_pool = configure_postgresql(&db_name).await;
        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        let redis_connection = Arc::new(RwLock::new(configure_redis()));
        let banned_token_store: BannedTokenStoreType =
            Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_connection)));
        let two_fa_store: TwoFACodeStoreType =
            Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));

        let email_client: EmailClientType = Arc::new(RwLock::new(MockEmailClient::default()));
        let cookie_jar = Arc::new(Jar::default());
        let app_state = AppState::new(
            user_store,
            banned_token_store.clone(),
            two_fa_store.clone(),
            email_client,
        );
        let app = Application::build(app_state, APP_ADDRESS)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        // Run the auth service in a separate async task
        // to avoid blocking the main test thread.
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone()) // Create a Reqwest http client instance
            .build()
            .unwrap();

        // Create new `TestApp` instance and return it
        TestApp {
            http_client,
            address,
            cookie_jar,
            banned_token_store,
            two_fa_store,
            db_name,
            clean_up_called: false,
        }
    }

    pub async fn clean_up(&mut self) {
        self.clean_up_called = true;
        delete_database(&self.db_name).await;
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    // pub async fn get_signup(&self) -> reqwest::Response {
    //     self.http_client
    //         .get(&format!("{}/signup", &self.address))
    //         .send()
    //         .await
    //         .expect("Failed to execute request.")
    // }

    // pub async fn get_login(&self) -> reqwest::Response {
    //     self.http_client
    //         .get(&format!("{}/login", &self.address))
    //         .send()
    //         .await
    //         .expect("Failed to execute request.")
    // }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    // pub async fn get_verify_2fa(&self) -> reqwest::Response {
    //     self.http_client
    //         .get(&format!("{}/verify_2fa", &self.address))
    //         .send()
    //         .await
    //         .expect("Failed to execute request.")
    // }

    // pub async fn get_verify_token(&self) -> reqwest::Response {
    //     self.http_client
    //         .get(&format!("{}/verify_token", &self.address))
    //         .send()
    //         .await
    //         .expect("Failed to execute request.")
    // }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

pub async fn configure_postgresql(db_name: &str) -> PgPool {
    let postgresql_conn_url = DATABASE_URL.to_owned();

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    // Create a new connection pool and return it
    get_postgres_pool(&postgresql_conn_url_with_db)
        .await
        .expect("Failed to create Postgres connection pool!")
}

pub async fn configure_database(db_conn_string: &str, db_name: &str) {
    // Create database connection
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // Connect to new database
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

pub async fn delete_database(db_name: &str) {
    let postgresql_conn_url: String = DATABASE_URL.to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Kill any active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}
