mod routes;
use std::error::Error;
use axum::{response::Html, routing::get, serve::Serve, Router};
use routes::*;
use tower_http::services::ServeDir;

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
    pub async fn build(address: &str) -> Result<Self, Box<dyn Error>> {
        // Move the Router definition from `main.rs` to here.
        // Also, remove the `hello` route.
        // We don't need it at this point!
        let router = Router::new()
            .route("/hello", get(hello_handler))
            .route("/login", get(login_handler))
            .route("/signup", get(signup_handler))
            .route("/logout", get(logout_handler))
            .route("/verify_2fa", get(verify_2fa_handler))
            .route("/verify_token", get(verify_token))
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
