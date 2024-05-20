use std::sync::{Arc, RwLock};

use auth_service::services::hashmap_user_store::HashMapUserStore;
use auth_service::Application;
use auth_service::app_state::{AppState, UserStoreType};
use reqwest::Client;
use uuid::Uuid;
pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store: UserStoreType = Arc::new(RwLock::new(HashMapUserStore::default()));
        let app_state = AppState::new(user_store);
        let app = Application::build(app_state, "127.0.0.1:0")
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        // Run the auth service in a separate async task
        // to avoid blocking the main test thread.
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let http_client = Client::new(); // Create a Reqwest http client instance

        // Create new `TestApp` instance and return it
        TestApp { http_client, address }
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

    pub async fn get_signup(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/signup", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_login(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/login", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_logout(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_verify_2fa(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/verify_2fa", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_verify_token(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/verify_token", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}
