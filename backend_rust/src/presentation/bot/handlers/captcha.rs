use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use shared_dtos::captcha::CaptchaBotResponse;

use crate::{
    errors::api::ApiResult, middlewares::verified_service::VerifiedService,
    services::captcha::CaptchaServiceTrait, state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(get_captcha))
}

#[utoipa::path(
    get,
    path = "/api/bot/captcha",
    tag = "Captcha",
    responses(
        (status = 200, description = "Generate captcha", body = CaptchaBotResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
async fn get_captcha(
    State(state): State<Arc<AppState>>,
    _service: VerifiedService,
) -> ApiResult<Json<CaptchaBotResponse>> {
    let captcha = state.captcha_service.get_captcha().await?;

    Ok(Json(CaptchaBotResponse {
        answer: captcha.answer,
        variants: captcha.variants,
        image_data: captcha.image_data,
    }))
}
