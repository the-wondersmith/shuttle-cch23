//! ### CCH 2023 Day 1 Solutions
//!

// Standard Library Imports
use core::ops::BitXor;
use std::collections::HashMap;

// Third-Party Imports
use axum::{
    async_trait,
    extract::{rejection::PathRejection, FromRequestParts, Json, Path},
    http::{request::Parts, StatusCode},
};
use serde_json::Value;

type NonNumericPacketIdResponse = (StatusCode, Json<HashMap<String, Vec<Value>>>);

// <editor-fold desc="// VariadicPathValues ...">

/// [`axum` extractor](axum::extract) for
/// variadic path values (e.g. `/endpoint/*values`)
#[derive(Debug)]
pub struct VariadicPathValues(pub Vec<Value>);

#[async_trait]
impl<State: Send + Sync> FromRequestParts<State> for VariadicPathValues {
    type Rejection = PathRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &State,
    ) -> anyhow::Result<Self, Self::Rejection> {
        let values = <Path<String> as FromRequestParts<State>>::from_request_parts(parts, state)
            .await?
            .split('/')
            .map(|part| {
                serde_json::from_str::<Value>(part)
                    .map_or_else(|_| Value::from(part), |value| value)
            })
            .collect::<Vec<Value>>();

        Ok(Self(values))
    }
}

// </editor-fold desc="// VariadicPathValues ...">

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

    /// Test that `calculate_sled_id`
    /// satisfies the conditions of [CCH 2023 Challenge 1](https://console.shuttle.rs/cch/challenge/1)
    #[rstest]
    #[case::challenge_example("/1/4/8", StatusCode::OK, b"1728")]
    #[case::bonus_example_one("/1/10", StatusCode::OK, b"1000")]
    #[case::bonus_example_two("/1/4/5/8/10", StatusCode::OK, b"27")]
    #[case::negative_packet_ids("/1/-12/45/-6", StatusCode::OK, b"42875")]
    #[case::mixed_packet_ids("/1/95/7552/sixty-four", StatusCode::BAD_REQUEST, b"0")]
    #[case::non_numeric_packet_ids("/1/fifty-five/fourteen", StatusCode::BAD_REQUEST, b"0")]
    #[test_log::test(tokio::test)]
    async fn test_challenge_one(
        service: TestService,
        #[case] url: &str,
        #[case] expected_status: StatusCode,
        #[case] expected_sled_id: &[u8],
    ) -> anyhow::Result<()> {
        let response = service.resolve(url).await?;

        assert_eq!(
            expected_status,
            response.status(),
            "status[expected: {}, actual: {}]",
            expected_status,
            response.status(),
        );

        let content = response.into_body().data().await.unwrap()?;

        Ok(assert!(
            content
                .eq_ignore_ascii_case(expected_sled_id)
                .bitor(url.split('/').all(|part| {
                    part.is_empty()
                        || part.parse::<i64>().is_ok()
                        || content
                            .as_ref()
                            .windows(part.len())
                            .any(|chunk| chunk == part.as_bytes())
                })),
            "content[expected: {}, actual: {}]",
            String::from_utf8_lossy(expected_sled_id),
            String::from_utf8_lossy(content.as_ref()),
        ))
    }
}
