pub mod app_state;
use axum::http::StatusCode;
pub mod domain;
pub mod routes;
pub mod services;
pub mod utils;
use app_state::AppState;
use axum::{
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    serve::Serve,
    Json, Router,
};
use domain::error::AuthAPIError;
use routes::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;
use axum::http::Method;

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
        // Allow the app service(running on our local machine and in production) to call the auth service
        let allowed_origins = [
            "http://localhost:8000".parse()?,
            // TODO: Replace [YOUR_DROPLET_IP] with your Droplet IP address
            "http://[YOUR_DROPLET_IP]:8000".parse()?,
        ];

        let cors = CorsLayer::new()
            // Allow GET and POST requests
            .allow_methods([Method::GET, Method::POST])
            // Allow cookies to be included in requests
            .allow_credentials(true)
            .allow_origin(allowed_origins);
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
            .layer(cors)
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
            AuthAPIError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid Token"),
            AuthAPIError::MissingToken => (StatusCode::BAD_REQUEST, "Missing Token"),
        };
        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });
        (status, body).into_response()
    }
}
