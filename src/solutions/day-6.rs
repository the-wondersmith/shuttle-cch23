//! ### CCH 2023 Day 6 Solutions
//!

// Third-Party Imports
use axum::extract::Json;

// Crate-Level Imports
use crate::types::ElfShelfCountSummary;

/// Complete [Day 6: Task + Bonus](https://console.shuttle.rs/cch/challenge/6#:~:text=🎄)
#[tracing::instrument(ret)]
pub async fn count_elves(text: String) -> Json<ElfShelfCountSummary> {
    Json(ElfShelfCountSummary::from(text))
}
