//! ### CCH 2023 Day 14 Solutions
//!

use std::collections::HashMap;

// Third-Party Imports
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use axum_template::TemplateEngine;

// Crate-Level Imports
use crate::state::ShuttleAppState;

/// Complete [Day 14: Task](https://console.shuttle.rs/cch/challenge/14#:~:text=⭐)
#[tracing::instrument(ret)]
pub async fn render_html_unsafe(
    Json(data): Json<HashMap<String, String>>,
) -> Result<String, StatusCode> {
    data.get("content")
        .ok_or(StatusCode::UNPROCESSABLE_ENTITY)
        .map(|content| {
            format!(
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/day-14/unsafe.tpl"
                )),
                content
            )
        })
}

/// Complete [Day 14: Bonus](https://console.shuttle.rs/cch/challenge/14#:~:text=🎁)
#[tracing::instrument(ret, skip(state))]
pub async fn render_html_safe(
    State(state): State<ShuttleAppState>,
    Json(data): Json<HashMap<String, String>>,
) -> Result<String, (StatusCode, String)> {
    state
        .templates
        .render("day-14/safe", data)
        .map_err(|error| (StatusCode::UNPROCESSABLE_ENTITY, format!("{error}")))
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
