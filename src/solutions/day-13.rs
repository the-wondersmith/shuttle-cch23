//! ### CCH 2023 Day 13 Solutions
//!

// Third-Party Imports
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use futures::prelude::*;
use serde_json::{Map as JsonObject, Value};

// Crate-Level Imports
use crate::types::{GiftOrder, ShuttleAppState};

/// Complete [Day 13: Task 1](https://console.shuttle.rs/cch/challenge/13#:~:text=⭐)
#[tracing::instrument(ret, skip(state))]
pub async fn simple_sql_select(
    State(state): State<ShuttleAppState>,
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
    State(state): State<ShuttleAppState>,
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
    State(state): State<ShuttleAppState>,
    Json(orders): Json<Vec<GiftOrder>>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    GiftOrder::insert_many(orders.iter(), &state.db)
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
    State(state): State<ShuttleAppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    GiftOrder::total_ordered(&state.db)
        .await
        .map(|count| {
            Json(Value::Object(JsonObject::from_iter([(
                "total".to_string(),
                Value::from(count),
            )])))
        })
        .map_err(|error| (StatusCode::FAILED_DEPENDENCY, format!("{error}")))
}

/// Complete [Day 13: Bonus](https://console.shuttle.rs/cch/challenge/13#:~:text=🎁)
#[tracing::instrument(ret, err(Debug), skip(state))]
pub async fn most_popular_gift(
    State(state): State<ShuttleAppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    GiftOrder::most_popular(&state.db)
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
