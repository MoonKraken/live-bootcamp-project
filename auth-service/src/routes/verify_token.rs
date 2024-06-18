use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::domain::error::AuthAPIError;
use crate::utils::auth::validate_token;
#[derive(Deserialize, Debug)]
pub struct VerifyTokenRequest {
    pub token: String,
}

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct VerifyTokenResponse {}

#[tracing::instrument(name = "Verify Token", skip_all)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    match validate_token(&request.token, &state.banned_token_store).await {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(e) => Err(AuthAPIError::InvalidToken),
    }
}
