#![forbid(unsafe_code)]
#![feature(impl_trait_in_assoc_type)]
#![cfg_attr(tarpaulin, feature(register_tool))]
#![cfg_attr(tarpaulin, register_tool(tarpaulin))]
#![cfg_attr(tarpaulin, feature(coverage_attribute))]
#![deny(missing_docs, missing_debug_implementations)]

//! # [`shuttle.rs`](https://shuttle.rs/) Christmas Code Hunt 2023
//!

// Module Declarations
#[cfg(test)]
mod tests;

pub mod types;
pub mod utils;

// Standard Library Imports
use core::ops::{Add, BitAnd, BitXor, Sub};
use std::collections::HashMap;

// Third-Party Imports
use axum::{
    body::Body,
    extract::{multipart::Multipart, Json, Path, State},
    http::{Request, StatusCode},
    response::IntoResponse,
    routing,
};
use chrono::{DateTime, Datelike, Utc};
use futures::prelude::*;
use image_rs::GenericImageView;
use serde_json::{Map as JsonObject, Value};
use shuttle_persist::{Persist, PersistInstance as Persistence};
use shuttle_secrets::{SecretStore, Secrets};
use shuttle_shared_db::Postgres as PgDb;
use tower::ServiceExt;
use tower_http::services::ServeFile;

// Crate-Level Imports

/// Run the project
#[cfg_attr(tarpaulin, coverage(off))]
#[cfg_attr(tarpaulin, tarpaulin::skip)]
#[tracing::instrument(skip_all)]
#[shuttle_runtime::main]
async fn main(
    #[PgDb] pool: sqlx::PgPool,
    #[Secrets] secrets: SecretStore,
    #[Persist] persistence: Persistence,
) -> shuttle_axum::ShuttleAxum {
    let state = types::ShuttleAppState::initialize(pool, Some(secrets), Some(persistence))?;

    Ok(router(state).into())
}

/// Create the project's main `Router` instance
#[tracing::instrument(skip(state))]
pub fn router(state: types::ShuttleAppState) -> routing::Router {
    routing::Router::new()
        .route("/", routing::get(hello_world))
        .route("/-1/error", routing::get(throw_error))
        .route("/1/*packets", routing::get(calculate_sled_id))
        .route("/4/contest", routing::post(summarize_reindeer_contest))
        .route("/4/strength", routing::post(calculate_reindeer_strength))
        .route("/6", routing::post(count_elves))
        .route(
            "/7/bake",
            routing::get(bake_cookies_from_recipe_and_pantry)
                .post(bake_cookies_from_recipe_and_pantry),
        )
        .route(
            "/7/decode",
            routing::get(decode_cookie_recipe).post(decode_cookie_recipe),
        )
        .route("/8/weight/:pokedex_id", routing::get(fetch_pokemon_weight))
        .route(
            "/8/drop/:pokedex_id",
            routing::get(calculate_pokemon_impact_momentum),
        )
        .route("/11/assets/:asset", routing::get(serve_static_asset))
        .route(
            "/11/red_pixels",
            routing::post(calculate_magical_red_pixel_count),
        )
        .route(
            "/12/save/:packet_it",
            routing::post(store_packet_id_timestamp),
        )
        .route(
            "/12/load/:packet_it",
            routing::get(retrieve_packet_id_timestamp),
        )
        .route("/12/ulids", routing::post(santas_ulid_hug_box))
        .route("/12/ulids/:weekday", routing::post(analyze_ulids))
        .route("/13/sql", routing::get(simple_sql_select))
        .route("/13/reset", routing::post(reset_db_schema))
        .route("/13/orders", routing::post(create_orders))
        .route("/13/orders/total", routing::get(total_order_count))
        .route("/13/orders/popular", routing::get(most_popular_gift))
        .with_state(state)
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
#[tracing::instrument(ret, skip(values), fields(left, right))]
pub async fn cube_the_bits(Path(values): Path<(u32, u32)>) -> Json<u32> {
    tracing::Span::current()
        .record("left", values.0)
        .record("right", values.1);
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
#[tracing::instrument(skip_all)]
pub async fn decode_cookie_recipe(
    types::CookieRecipeHeader(recipe): types::CookieRecipeHeader<Value>,
) -> Json<Value> {
    Json(recipe)
}

/// Complete [Day 7: Bonus](https://console.shuttle.rs/cch/challenge/7#:~:text=🎁)
#[tracing::instrument(skip_all, fields(request, response))]
pub async fn bake_cookies_from_recipe_and_pantry(
    types::CookieRecipeHeader(data): types::CookieRecipeHeader<types::CookieRecipeInventory>,
) -> types::RecipeAnalysisResponse {
    tracing::Span::current().record("request", format!("{}", &data).as_str());

    let data = data.bake();

    tracing::Span::current().record("response", format!("{}", &data).as_str());

    tracing::info!("\u{1F6CE}\u{FE0F}\u{3000}\u{1F36A}");

    (StatusCode::OK, Json(data))
}

/// Complete [Day 8: Challenge](https://console.shuttle.rs/cch/challenge/8#:~:text=⭐)
#[tracing::instrument(ret)]
pub async fn fetch_pokemon_weight(
    Path(pokedex_id): Path<u16>,
) -> Result<Json<f64>, (StatusCode, String)> {
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
    let momentum = poke_weight * final_speed;

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

/// Endpoint 1/2 for [Day 12: Challenge](https://console.shuttle.rs/cch/challenge/12#:~:text=⭐)
#[tracing::instrument(ret, skip(state), fields(new, old))]
pub async fn store_packet_id_timestamp(
    Path(packet_id): Path<String>,
    State(state): State<types::ShuttleAppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .persistence
        .save(&packet_id, Utc::now())
        .map(|()| StatusCode::OK)
        .map_err(|error| (StatusCode::FAILED_DEPENDENCY, format!("{error}")))
}

/// Endpoint 2/2 for [Day 12: Challenge](https://console.shuttle.rs/cch/challenge/12#:~:text=⭐)
#[tracing::instrument(ret, skip(state), fields(new, old))]
pub async fn retrieve_packet_id_timestamp(
    Path(packet_id): Path<String>,
    State(state): State<types::ShuttleAppState>,
) -> Result<Json<u64>, (StatusCode, String)> {
    let now = Utc::now();

    state
        .persistence
        .load::<DateTime<Utc>>(&packet_id)
        .or_else(|_| state.persistence.save(&packet_id, now).map(|()| now))
        .map(|stamp| Json(now.sub(stamp).num_seconds().unsigned_abs()))
        .map_err(|error| (StatusCode::FAILED_DEPENDENCY, format!("{error}")))
}

/// Complete [Day 12: Bonus 1](https://console.shuttle.rs/cch/challenge/12#:~:text=🎁)
#[tracing::instrument(ret)]
pub async fn santas_ulid_hug_box(Json(ulids): Json<Vec<ulid::Ulid>>) -> Json<Vec<uuid::Uuid>> {
    Json(
        ulids
            .into_iter()
            .rev()
            .map(<uuid::Uuid as From<ulid::Ulid>>::from)
            .collect::<Vec<uuid::Uuid>>(),
    )
}

/// Complete [Day 12: Bonus 2](https://console.shuttle.rs/cch/challenge/12#:~:text=🎁)
///
/// For the set of provided ULIDs, returns a cumulative count of:
///   - How many have entropy bits where the Least Significant Bit (LSB) is 1?
///   - How many of the ULIDs were generated on a Christmas Eve? (day == 24) (?)
///   - How many were generated on a <weekday>? (A number in the path between 0 (Monday) and 6 (Sunday))
///   - How many were generated in the future? (has a date later than the current time)
#[tracing::instrument(ret)]
pub async fn analyze_ulids(
    Path(weekday): Path<u32>,
    Json(ulids): Json<Vec<ulid::Ulid>>,
) -> Json<JsonObject<String, Value>> {
    let now = Utc::now();
    let (mut chaotic, mut xmas_eve, mut in_future, mut on_weekday) = (0u64, 0u64, 0u64, 0u64);

    for id in ulids {
        let created_at: DateTime<Utc> = id.datetime().into();

        if now < created_at {
            in_future += 1;
        }

        if id.random().bitand(1) == 1 {
            chaotic += 1;
        }

        if created_at.day() == 24 && created_at.month() == 12 {
            xmas_eve += 1;
        }

        if created_at.weekday().num_days_from_monday() == weekday {
            on_weekday += 1;
        }
    }

    Json(JsonObject::<String, Value>::from_iter(
        [
            ("LSB is 1".to_string(), Value::from(chaotic)),
            ("weekday".to_string(), Value::from(on_weekday)),
            ("christmas eve".to_string(), Value::from(xmas_eve)),
            ("in the future".to_string(), Value::from(in_future)),
        ]
        .into_iter(),
    ))
}

/// Complete [Day 13: Task 1](https://console.shuttle.rs/cch/challenge/13#:~:text=⭐)
#[tracing::instrument(ret, skip(state))]
pub async fn simple_sql_select(
    State(state): State<types::ShuttleAppState>,
) -> Result<Json<i32>, (StatusCode, String)> {
    sqlx::query_scalar::<_, i32>("SELECT 20231213")
        .fetch_one(&state.db)
        .await
        .map_err(|error| (StatusCode::EXPECTATION_FAILED, format!("{error}")))
        .map(Json)
}

/// Endpoint 1/3 for [Day 13: Task 2](https://console.shuttle.rs/cch/challenge/13#:~:text=⭐)
#[tracing::instrument(ret, err(Debug), skip(state))]
pub async fn reset_db_schema(
    State(state): State<types::ShuttleAppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DROP TABLE IF EXISTS orders;")
        .execute(&state.db)
        .and_then(|_| {
            sqlx::query(
                r#"CREATE TABLE IF NOT EXISTS orders (
                 id INT PRIMARY KEY,
                 gift_name VARCHAR(50),
                 quantity INT,
                 region_id INT
               );
            "#,
            )
            .execute(&state.db)
        })
        .await
        .map(|_| StatusCode::OK)
        .map_err(|error| (StatusCode::FAILED_DEPENDENCY, format!("{error}")))
}

/// Endpoint 2/3 for [Day 13: Task 2](https://console.shuttle.rs/cch/challenge/13#:~:text=⭐)
#[tracing::instrument(ret, err(Debug), skip_all, fields(orders.count = orders.len()))]
pub async fn create_orders(
    State(state): State<types::ShuttleAppState>,
    Json(orders): Json<Vec<types::GiftOrder>>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    types::GiftOrder::insert_many(orders.iter(), &state.db)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|error| {
            (
                StatusCode::FAILED_DEPENDENCY,
                Json(Value::Object(JsonObject::<String, Value>::from_iter([
                    ("error".to_string(), Value::String(format!("{error}"))),
                    ("request".to_string(), serde_json::to_value(orders).unwrap()),
                ]))),
            )
        })
}

/// Endpoint 3/3 for [Day 13: Task 2](https://console.shuttle.rs/cch/challenge/13#:~:text=⭐)
#[tracing::instrument(ret, err(Debug), skip(state))]
pub async fn total_order_count(
    State(state): State<types::ShuttleAppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    types::GiftOrder::total_ordered(&state.db)
        .await
        .map(|count| {
            Json(Value::Object(JsonObject::from_iter([(
                "total".to_string(),
                Value::from(count),
            )])))
        })
        .map_err(|error| (StatusCode::FAILED_DEPENDENCY, format!("{error}")))
}

/// Complete [Day 13: Task 3](https://console.shuttle.rs/cch/challenge/13#:~:text=⭐)
#[tracing::instrument(ret, err(Debug), skip(state))]
pub async fn most_popular_gift(
    State(state): State<types::ShuttleAppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    types::GiftOrder::most_popular(&state.db)
        .await
        .map(|count| {
            Json(Value::Object(JsonObject::from_iter([(
                "popular".to_string(),
                match count {
                    None => Value::Null,
                    Some((toy, _)) => Value::String(toy),
                },
            )])))
        })
        .map_err(|error| (StatusCode::FAILED_DEPENDENCY, format!("{error}")))
}
