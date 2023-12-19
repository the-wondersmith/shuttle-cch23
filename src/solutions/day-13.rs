//! ### CCH 2023 Day 13 Solutions
//!

// Third-Party Imports
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonObject, Value};
use sqlx::{error::Error as DbError, postgres::PgQueryResult};

// Crate-Level Imports
use crate::state::ShuttleAppState;

// <editor-fold desc="// GiftOrder ...">

/// A gift order
#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GiftOrder {
    /// the order's sequential id
    pub id: i64,
    /// how many `{gift_name}`s were ordered
    pub quantity: i64,
    /// the gift's elf-readable name
    pub gift_name: String,
    /// the region to which the
    /// gift must be delivered
    pub region_id: i64,
}

impl GiftOrder {
    /// ...
    pub async fn insert(&self, db: &sqlx::PgPool) -> Result<PgQueryResult, DbError> {
        Self::insert_many([self].into_iter(), db).await
    }

    /// ...
    pub async fn insert_many<'orders, Orders: Iterator<Item = &'orders Self>>(
        orders: Orders,
        db: &sqlx::PgPool,
    ) -> Result<PgQueryResult, DbError> {
        sqlx::QueryBuilder::<sqlx::Postgres>::new(
            "INSERT INTO ORDERS (id, quantity, gift_name, region_id) ",
        )
        .push_values(orders, |mut builder, order| {
            builder
                .push_bind(order.id)
                .push_bind(order.quantity)
                .push_bind(order.gift_name.clone())
                .push_bind(order.region_id);
        })
        .build()
        .execute(db)
        .await
    }

    /// ...
    pub async fn total_ordered(db: &sqlx::PgPool) -> Result<i64, DbError> {
        sqlx::query_scalar::<_, i64>("SELECT SUM(quantity) FROM orders")
            .fetch_one(db)
            .await
    }

    /// ...
    pub async fn most_popular(db: &sqlx::PgPool) -> Result<Option<(String, i64)>, DbError> {
        sqlx::query_as(
            r#"
            SELECT
                gift_name,
                SUM(quantity) as popularity
            FROM
                orders
            GROUP BY
                gift_name
            ORDER BY
                popularity
            DESC
            LIMIT 1
        "#,
        )
        .fetch_optional(db)
        .await
    }
}

// </editor-fold desc="// GiftOrder ...">

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
pub async fn reset_day_13_schema(
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
    if !orders.is_empty() {
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
    } else {
        Ok(StatusCode::OK)
    }
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

#[cfg(test)]
mod tests {
    //! ## I/O-free Unit Tests

    #![allow(unused_imports, clippy::unit_arg)]

    // Standard Library Imports
    use core::{cmp::PartialEq, fmt::Debug, ops::BitOr, str::FromStr};
    use std::collections::HashMap;

    // Third-Party Imports
    use axum::{
        body::{Body, BoxBody, HttpBody},
        http::{
            header as headers,
            request::{Builder, Parts},
            Method, Request, Response, StatusCode,
        },
        routing::Router,
    };
    use once_cell::sync::Lazy;
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
    use rstest::{fixture, rstest};
    use serde_json::{error::Error as SerdeJsonError, Value};
    use shuttle_shared_db::Postgres as ShuttleDB;
    use tower::{MakeService, ServiceExt};

    // Crate-Level Imports
    use crate::utils::{service, TestService};
}
