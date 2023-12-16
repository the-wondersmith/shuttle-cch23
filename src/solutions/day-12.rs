//! ### CCH 2023 Day 12 Solutions
//!

// Standard Library Imports
use core::ops::{BitAnd, Sub};

// Third-Party Imports
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Datelike, Utc};
use serde_json::{Map as JsonObject, Value};

// Crate-Level Imports
use crate::types::ShuttleAppState;

/// Endpoint 1/2 for [Day 12: Challenge](https://console.shuttle.rs/cch/challenge/12#:~:text=⭐)
#[tracing::instrument(ret, skip(state), fields(new, old))]
pub async fn store_packet_id_timestamp(
    Path(packet_id): Path<String>,
    State(state): State<ShuttleAppState>,
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
    State(state): State<ShuttleAppState>,
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
