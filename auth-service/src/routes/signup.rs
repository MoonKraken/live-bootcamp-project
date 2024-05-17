use axum::response::IntoResponse;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

pub async fn signup_handler(Json(request): Json<SignupRequest>) -> impl IntoResponse {
    StatusCode::OK.into_response()
}

#[derive(Deserialize, Debug)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}
