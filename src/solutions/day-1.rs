//! ### CCH 2023 Day 1 Solutions
//!

// Standard Library Imports
use core::ops::BitXor;
use std::collections::HashMap;

// Third-Party Imports
use axum::{
    extract::{Json, Path},
    http::StatusCode,
};
use serde_json::Value;

// Crate-Level Imports
use crate::types::{NonNumericPacketIdResponse, VariadicPathValues};

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
    VariadicPathValues(packets): VariadicPathValues,
) -> Result<Json<i64>, NonNumericPacketIdResponse> {
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
