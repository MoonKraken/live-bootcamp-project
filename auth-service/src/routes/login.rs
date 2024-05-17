use axum::http::StatusCode;
use axum::response::IntoResponse;
pub async fn login_handler() -> impl IntoResponse {
    // Update this to a custom message!
    StatusCode::OK.into_response()
}
