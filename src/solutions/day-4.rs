//! ### CCH 2023 Day 4 Solutions
//!

// Standard Library Imports
use core::ops::Add;
use std::collections::HashMap;

// Third-Party Imports
use axum::extract::Json;

// Crate-Level Imports
use crate::types::ReindeerStats;

/// Complete [Day 4: Challenge](https://console.shuttle.rs/cch/challenge/4#:~:text=⭐)
#[tracing::instrument(ret)]
pub async fn calculate_reindeer_strength(Json(stats): Json<Vec<ReindeerStats>>) -> Json<i64> {
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
    Json(stats): Json<Vec<ReindeerStats>>,
) -> Json<HashMap<String, String>> {
    Json(ReindeerStats::summarize(&stats))
}
