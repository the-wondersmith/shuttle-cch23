#![allow(unused_imports)]
//! ### CCH 2023 Day 18 Solutions
//!

// Standard Library Imports
use core::ops::{Add, BitAnd, BitXor, Sub};
use std::collections::HashMap;
use std::ops::BitOr;

// Third-Party Imports
use axum::{
    body::Body,
    extract::{multipart::Multipart, Json, Path, State},
    http::{Request, StatusCode},
    response::IntoResponse,
    routing,
};
use axum_template::TemplateEngine;
use chrono::{DateTime, Datelike, Utc};
use futures::prelude::*;
use image_rs::GenericImageView;
use itertools::Itertools;
use num_traits::cast::FromPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonObject, Value};
use shuttle_persist::{Persist, PersistInstance as Persistence};
use shuttle_secrets::{SecretStore, Secrets};
use shuttle_shared_db::Postgres as PgDb;
use sqlx::{error::Error as DbError, postgres::PgQueryResult, FromRow};
use tower::ServiceExt;
use tower_http::services::ServeFile;
use unicode_normalization::UnicodeNormalization;

// Crate-Level Imports
use crate::solutions::day_13::GiftOrder;
use crate::state::ShuttleAppState;
use crate::{state, utils};

/// A summary of gift order totals for a
/// given geographical region
#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug, Default, FromRow, Serialize, Deserialize)]
pub struct RegionalOrderTotal {
    /// the region's elf-readable name
    #[sqlx(rename = "name")]
    pub region: String,
    /// the total number of the region's
    /// associated gift orders
    #[serde(rename = "total")]
    pub total_orders: i64,
}

// <editor-fold desc="// RegionalTopGifts ...">

/// A list of the most popular gifts
/// in a given geographical region
#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug, Default, FromRow, Serialize, Deserialize)]
pub struct RegionalTopGifts {
    /// the region's elf-readable name
    pub region: String,
    /// the elf-readable names of the
    /// top N gifts in the region
    pub top_gifts: Vec<String>,
}

// </editor-fold desc="// RegionalTopGifts ...">

// <editor-fold desc="// GiftOrderRegion ...">

/// The geographical region a gift order
/// originated from or will be delivered to
#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug, Default, FromRow, Serialize, Deserialize)]
pub struct GiftOrderRegion {
    /// the region's sequential id
    pub id: i64,
    /// the region's elf-readable name
    pub name: String,
}

impl GiftOrderRegion {
    /// ...
    pub async fn insert(&self, db: &sqlx::PgPool) -> Result<PgQueryResult, DbError> {
        Self::insert_many([self].into_iter(), db).await
    }

    /// ...
    pub async fn insert_many<'orders, Orders: Iterator<Item = &'orders Self>>(
        orders: Orders,
        db: &sqlx::PgPool,
    ) -> Result<PgQueryResult, DbError> {
        sqlx::QueryBuilder::<sqlx::Postgres>::new("INSERT INTO regions (id, name) ")
            .push_values(orders, |mut builder, region| {
                builder.push_bind(region.id).push_bind(region.name.clone());
            })
            .build()
            .execute(db)
            .await
    }

    /// ...
    pub async fn total_orders_by_region(
        db: &sqlx::PgPool,
    ) -> Result<Vec<RegionalOrderTotal>, DbError> {
        sqlx::query_as::<_, RegionalOrderTotal>(
            r#"SELECT
              regions.name,
              SUM(orders.quantity) AS total_orders
            FROM
              regions
            INNER JOIN
              orders ON regions.id = orders.region_id
            GROUP BY
              regions.name
            ORDER BY
              regions.name ASC"#,
        )
        .fetch_all(db)
        .await
    }

    /// ...
    pub async fn top_n_most_popular(
        number: u64,
        db: &sqlx::PgPool,
    ) -> Result<Vec<RegionalTopGifts>, DbError> {
        sqlx::query_as::<sqlx::Postgres, RegionalTopGifts>(
            r#"
            WITH ranked_gifts AS (
              SELECT
                regions.name AS region_name,
                orders.gift_name,
                ROW_NUMBER() OVER (
                  PARTITION BY regions.name
                  ORDER BY
                    SUM(orders.quantity) DESC,
                    orders.gift_name ASC
                ) AS row_number
              FROM
                regions
                LEFT JOIN orders ON regions.id = orders.region_id
              GROUP BY
                regions.name,
                orders.gift_name
            )
            SELECT
              region_name AS "region",
              (ARRAY_REMOVE(
                ARRAY_AGG(
                  gift_name
                  ORDER BY
                    row_number
                ),
                NULL
              ))[0:$1] AS "top_gifts"
            FROM
              ranked_gifts
            GROUP BY
              region_name
            ORDER BY
              region_name ASC;
            "#,
        )
        .bind(number as i64)
        .fetch_all(db)
        .await
    }
}

// </editor-fold desc="// GiftOrderRegion ...">

/// Endpoint 1/3 for [Day 18: Task 1](https://console.shuttle.rs/cch/challenge/18#:~:text=⭐)
#[tracing::instrument(ret, err(Debug), skip(state))]
pub async fn reset_day_18_schema(
    State(state): State<ShuttleAppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DROP TABLE IF EXISTS orders;")
        .execute(&state.db)
        .and_then(|_| sqlx::query("DROP TABLE IF EXISTS regions;").execute(&state.db))
        .and_then(|_| {
            sqlx::query(
                r#"CREATE TABLE regions (
                  id INT PRIMARY KEY,
                  name VARCHAR(50)
                );
            "#,
            )
            .execute(&state.db)
        })
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

/// Endpoint 2/3 for [Day 18: Task 1](https://console.shuttle.rs/cch/challenge/18#:~:text=⭐)
#[tracing::instrument(ret, err(Debug), skip_all, fields(regions.count = regions.len()))]
pub async fn create_regions(
    State(state): State<ShuttleAppState>,
    Json(regions): Json<Vec<GiftOrderRegion>>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    if !regions.is_empty() {
        GiftOrderRegion::insert_many(regions.iter(), &state.db)
            .await
            .map(|_| StatusCode::OK)
            .map_err(|error| {
                (
                    StatusCode::FAILED_DEPENDENCY,
                    Json(Value::Object(JsonObject::<String, Value>::from_iter([
                        ("error".to_string(), Value::String(format!("{error}"))),
                        (
                            "request".to_string(),
                            serde_json::to_value(regions).unwrap(),
                        ),
                    ]))),
                )
            })
    } else {
        Ok(StatusCode::OK)
    }
}

/// Endpoint 3/3 for [Day 18: Task 1](https://console.shuttle.rs/cch/challenge/18#:~:text=⭐)
#[tracing::instrument(ret, err(Debug), skip(state))]
pub async fn get_order_count_by_region(
    State(state): State<ShuttleAppState>,
) -> Result<Json<Vec<RegionalOrderTotal>>, (StatusCode, String)> {
    GiftOrderRegion::total_orders_by_region(&state.db)
        .await
        .map(Json)
        .map_err(|error| (StatusCode::FAILED_DEPENDENCY, format!("{error}")))
}

/// Complete [Day 18: Bonus](https://console.shuttle.rs/cch/challenge/18#:~:text=🎁)
#[tracing::instrument(ret, err(Debug), skip(state))]
pub async fn get_top_n_gifts_by_region(
    State(state): State<ShuttleAppState>,
    Path(number): Path<u64>,
) -> Result<Json<Vec<RegionalTopGifts>>, (StatusCode, String)> {
    GiftOrderRegion::top_n_most_popular(number, &state.db)
        .await
        .map(Json)
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
