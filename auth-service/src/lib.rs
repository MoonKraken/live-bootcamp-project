pub mod app_state;
use axum::http::StatusCode;
pub mod routes;
pub mod services;
pub mod domain;
pub mod utils;
use std::error::Error;
use axum::{response::{Html, IntoResponse, Response}, routing::{get, post}, serve::Serve, Json, Router};
use domain::error::AuthAPIError;
use routes::*;
use serde::{Deserialize, Serialize};
use tower_http::services::ServeDir;
use app_state::AppState;

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<Router, Router>,
    // address is exposed as a public field
    // so we have access to it in tests.
    pub address: String,
}

async fn hello_handler() -> Html<&'static str> {
    // Update this to a custom message!
    Html("<h1>Hello, My name is MoonKraken!</h1>")
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        // Move the Router definition from `main.rs` to here.
        // Also, remove the `hello` route.
        // We don't need it at this point!
        let router = Router::new()
            .route("/hello", get(hello_handler))
            .route("/login", post(login_handler))
            .route("/signup", get(signup_handler))
            .route("/signup", post(signup_handler))
            .route("/logout", get(logout_handler))
            .route("/verify_2fa", get(verify_2fa_handler))
            .route("/verify_token", get(verify_token))
            .with_state(app_state)
            .nest_service("/", ServeDir::new("assets"));
        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);
        // Create a new Application instance and return it
        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::IncorrectCredentials => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AuthAPIError::UnexpectedError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
            }
        };
        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });
        (status, body).into_response()
    }
}
