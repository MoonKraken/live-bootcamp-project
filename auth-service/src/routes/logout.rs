use axum::response::IntoResponse;
use axum::http::StatusCode;
pub async fn logout_handler() -> impl IntoResponse {
    // Update this to a custom message!
    StatusCode::OK.into_response()
}
