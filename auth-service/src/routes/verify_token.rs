use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::domain::error::AuthAPIError;
use crate::utils::auth::validate_token;
#[derive(Deserialize, Debug)]
pub struct VerifyTokenRequest {
    pub token: String,
}

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct VerifyTokenResponse {}
pub async fn verify_token(
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    match validate_token(&request.token).await {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(_) => Err(AuthAPIError::InvalidToken),
    }
}
