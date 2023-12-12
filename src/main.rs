#![forbid(unsafe_code)]
#![cfg_attr(tarpaulin, feature(register_tool))]
#![cfg_attr(tarpaulin, register_tool(tarpaulin))]
#![cfg_attr(tarpaulin, feature(coverage_attribute))]
#![deny(missing_docs, missing_debug_implementations)]

//! # [`shuttle.rs`](https://shuttle.rs/) Christmas Code Hunt 2023
//!

// Standard Library Imports
use core::ops::{Add, BitXor};
use std::collections::HashMap;

// Third-Party Imports
use axum::{
    body::Body,
    extract::{Json, Path},
    http::{Request, StatusCode},
    response::IntoResponse,
    routing,
};
use serde_json::Value;
use tower::ServiceExt;
use tower_http::services::ServeFile;

// Module Declarations
#[cfg(test)]
mod tests;

pub mod types;
pub mod utils;

/// Run the project
#[cfg_attr(tarpaulin, coverage(off))]
#[cfg_attr(tarpaulin, tarpaulin::skip)]
#[tracing::instrument]
#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    Ok(router().into())
}

/// Create the project's main `Router` instance
#[tracing::instrument]
pub fn router() -> routing::Router {
    routing::Router::new()
        .route("/", routing::get(hello_world))
        .route("/-1/error", routing::get(throw_error))
        .route("/1/*packets", routing::get(calculate_sled_id))
        .route("/4/contest", routing::post(summarize_reindeer_contest))
        .route("/4/strength", routing::post(calculate_reindeer_strength))
        .route("/6", routing::post(count_elves))
        .route("/7/bake", routing::get(analyze_recipe).post(analyze_recipe))
        .route("/7/decode", routing::get(decode_cookie).post(decode_cookie))
        .route("/8/weight/:pokedex_id", routing::get(fetch_pokemon_weight))
        .route(
            "/8/drop/:pokedex_id",
            routing::get(calculate_pokemon_impact_momentum),
        )
        .route("/11/assets/:asset", routing::get(serve_static_asset))
        // .nest_service("/11/assets", ServeDir::new("assets"))
        .route(
            "/11/red_pixels",
            routing::post(calculate_magical_red_pixel_count),
        )
}

/// Complete [Day -1: Challenge](https://console.shuttle.rs/cch/challenge/-1#:~:text=⭐)
#[tracing::instrument(ret)]
pub async fn hello_world() -> &'static str {
    "Hello Shuttle CCH 2023!"
}

/// Complete [Day -1: Bonus](https://console.shuttle.rs/cch/challenge/-1#:~:text=🎁)
#[tracing::instrument(ret)]
pub async fn throw_error() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Gimme them bonus points")
}

/// Complete [Day 1: Challenge](https://console.shuttle.rs/cch/challenge/1#:~:text=⭐)
#[allow(dead_code)]
#[cfg_attr(tarpaulin, coverage(off))]
#[cfg_attr(tarpaulin, tarpaulin::skip)]
#[tracing::instrument(ret)]
pub async fn cube_the_bits(Path(values): Path<(u32, u32)>) -> Json<u32> {
    Json(BitXor::bitxor(values.0, values.1).pow(3u32))
}

/// Complete [Day 1: Bonus](https://console.shuttle.rs/cch/challenge/1#:~:text=🎁)
#[tracing::instrument(ret)]
pub async fn calculate_sled_id(
    types::VariadicPathValues(packets): types::VariadicPathValues,
) -> Result<Json<i64>, types::NonNumericPacketIdResponse> {
    let (mut packet_ids, mut invalid_packets) = (Vec::<Value>::new(), Vec::<Value>::new());

    for value in packets {
        if matches!(value, Value::Number(_)) {
            packet_ids.push(value);
        } else {
            invalid_packets.push(value);
        }
    }

    if invalid_packets.is_empty() {
        Ok(Json(
            packet_ids
                .iter()
                .filter_map(Value::as_i64)
                .fold(0i64, BitXor::bitxor)
                .pow(3u32),
        ))
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            Json(HashMap::from([(
                String::from("non-numeric packet ids"),
                invalid_packets,
            )])),
        ))
    }
}

/// Complete [Day 4: Challenge](https://console.shuttle.rs/cch/challenge/4#:~:text=⭐)
#[tracing::instrument(ret)]
pub async fn calculate_reindeer_strength(
    Json(stats): Json<Vec<types::ReindeerStats>>,
) -> Json<i64> {
    Json(
        stats
            .iter()
            .map(|reindeer| reindeer.strength)
            .fold(0i64, i64::add),
    )
}

/// Complete [Day 4: Bonus](https://console.shuttle.rs/cch/challenge/4#:~:text=🎁)
#[tracing::instrument(ret)]
pub async fn summarize_reindeer_contest(
    Json(stats): Json<Vec<types::ReindeerStats>>,
) -> Json<HashMap<String, String>> {
    Json(types::ReindeerStats::summarize(&stats))
}

/// Complete [Day 6: Task + Bonus](https://console.shuttle.rs/cch/challenge/6#:~:text=🎄)
#[tracing::instrument(ret)]
pub async fn count_elves(text: String) -> Json<types::ElfShelfCountSummary> {
    Json(types::ElfShelfCountSummary::from(text))
}

/// Complete [Day 7: Challenge](https://console.shuttle.rs/cch/challenge/7#:~:text=⭐)
#[tracing::instrument(ret)]
pub async fn decode_cookie(
    types::CookieRecipeHeader(recipe): types::CookieRecipeHeader<Value>,
) -> Json<Value> {
    Json(recipe)
}

/// Complete [Day 7: Bonus](https://console.shuttle.rs/cch/challenge/7#:~:text=🎁)
#[tracing::instrument(ret)]
pub async fn analyze_recipe(
    types::CookieRecipeHeader(data): types::CookieRecipeHeader<types::CookieRecipeInventory>,
) -> Result<Json<types::CookieRecipeInventory>, types::EmptyRecipeOrPantryResponse> {
    if data.cookies != 0 || data.recipe.is_empty() || data.pantry.is_empty() {
        Err((StatusCode::UNPROCESSABLE_ENTITY, Json(data)))
    } else {
        Ok(Json(data.bake()))
    }
}

/// Complete [Day 8: Challenge](https://console.shuttle.rs/cch/challenge/8#:~:text=⭐)
#[tracing::instrument(ret)]
pub async fn fetch_pokemon_weight(
    Path(pokedex_id): Path<u16>,
) -> Result<Json<u32>, (StatusCode, String)> {
    Ok(Json(utils::fetch_pokemon_weight(pokedex_id).await?))
}

/// Complete [Day 8: Bonus](https://console.shuttle.rs/cch/challenge/8#:~:text=🎁)
#[allow(non_upper_case_globals)]
#[tracing::instrument(ret)]
pub async fn calculate_pokemon_impact_momentum(
    Path(pokedex_id): Path<u16>,
) -> Result<Json<f64>, (StatusCode, String)> {
    /// Gravitational acceleration in m/s²
    const gravity: f64 = 9.825;
    /// Chimney height in meters
    const drop_height: f64 = 10.0;

    let poke_weight = utils::fetch_pokemon_weight(pokedex_id).await?;

    // Calculate the final speed with kinematic equation
    let final_speed = (2.0 * gravity * drop_height).sqrt();

    // Calculate the impact momentum
    let momentum = f64::from(poke_weight) * final_speed;

    Ok(Json(momentum))
}

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
#[tracing::instrument(ret)]
pub async fn calculate_magical_red_pixel_count() -> impl IntoResponse {
    todo!()
}
