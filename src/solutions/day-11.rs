//! ### CCH 2023 Day 11 Solutions
//!

// Third-Party Imports
use axum::{
    body::Body,
    extract::{multipart::Multipart, Json, Path},
    http::{Request, StatusCode},
    response::IntoResponse,
};
use image_rs::GenericImageView;
use tower::ServiceExt;
use tower_http::services::ServeFile;

// Crate-Level Imports
use crate::utils;

/// Complete [Day 11: Challenge](https://console.shuttle.rs/cch/challenge/11#:~:text=⭐)
#[tracing::instrument(skip_all, fields(error))]
pub async fn serve_static_asset(
    Path(asset): Path<String>,
    request: Request<Body>,
) -> impl IntoResponse {
    ServeFile::new(format!("assets/{asset}"))
        .oneshot(request)
        .await
        .map(|response| {
            match response.status() {
                StatusCode::OK => tracing::info!("resolved asset for: {asset}"),
                StatusCode::NOT_FOUND => tracing::warn!("no asset found for: {asset}"),
                status => tracing::error!(
                    r#"error resolving asset: {{"asset": {asset}, "status": {status}}}"#
                ),
            };

            IntoResponse::into_response(response)
        })
        .map_err(|error| {
            tracing::Span::current().record("error", &error.to_string());
            StatusCode::UNPROCESSABLE_ENTITY
        })
}

/// Complete [Day 11: Bonus](https://console.shuttle.rs/cch/challenge/11#:~:text=🎁)
#[tracing::instrument(skip(request), fields(image.name, image.magic.red))]
pub async fn calculate_magical_red_pixel_count(
    mut request: Multipart,
) -> Result<Json<u64>, StatusCode> {
    let field = request
        .next_field()
        .await
        .map_err(|error| {
            tracing::error!("{error:?}");
            StatusCode::UNPROCESSABLE_ENTITY
        })?
        .ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;

    tracing::Span::current().record("image.name", field.name().unwrap());

    let image = field
        .bytes()
        .await
        .map_err(|error| {
            tracing::error!("{error:?}");
        })
        .and_then(|data| {
            image_rs::load_from_memory(data.as_ref()).map_err(|error| {
                tracing::error!("{error:?}");
            })
        })
        .map_err(|()| StatusCode::UNPROCESSABLE_ENTITY)?;

    let magic_red_count = image
        .pixels()
        .map(utils::is_magic_red)
        .map(u64::from)
        .sum::<u64>();

    tracing::Span::current().record("image.magic.red", magic_red_count);

    Ok(Json(magic_red_count))
}
