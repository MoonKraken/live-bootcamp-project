use axum::response::IntoResponse;
use axum::http::StatusCode;
pub async fn verify_2fa_handler() -> impl IntoResponse {
    // Update this to a custom message!
    StatusCode::OK.into_response()
}
