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
use crate::types::ShuttleAppState;

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
