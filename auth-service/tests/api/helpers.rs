use auth_service::Application;
use reqwest::Client;

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let app = Application::build("127.0.0.1:0")
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