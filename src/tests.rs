//! ## I/O-free Unit Tests

#![allow(unused_imports, clippy::unit_arg)]

// Standard Library Imports

// Third-Party Imports
use axum::body::HttpBody;
use axum::{
    body::{Body, BoxBody},
    http::{Request, Response, StatusCode},
};
use http_body_util::BodyExt;
use once_cell::sync::OnceCell;
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
use rstest::rstest;
use tower::ServiceExt;

// Crate-Level Imports
use super::{calculate_sled_id, cube_the_bits, hello_world, router, throw_error};

// <editor-fold desc="// Constants ...">

// </editor-fold desc="// Constants ...">

// <editor-fold desc="// Fixtures ...">

// </editor-fold desc="// Fixtures ...">

// <editor-fold desc="// Utility Functions ...">

/// Bounce the supplied request body off the project's
/// `axum::Router` at the specified path and return the
/// resolved response
async fn resolve(uri: &str, body: Option<Body>) -> anyhow::Result<Response<BoxBody>> {
    let request = Request::builder()
        .uri(uri)
        .body(body.unwrap_or(Body::empty()))?;
    Ok(router().oneshot(request).await?)
}

// </editor-fold desc="// Utility Functions ...">

// <editor-fold desc="// Tests ...">

/// Test that `hello_world` and `throw_error`
/// satisfy the conditions of [CCH 2023 Challenge -1](https://console.shuttle.rs/cch/challenge/-1)
#[rstest]
#[case::hello_world("/", StatusCode::OK, b"Hello Shuttle CCH 2023!")]
#[case::throw_error(
    "/-1/error",
    StatusCode::INTERNAL_SERVER_ERROR,
    b"Gimme them bonus points"
)]
#[test_log::test(tokio::test)]
async fn test_challenge_minus_one(
    #[case] url: &str,
    #[case] status: StatusCode,
    #[case] content: &[u8],
) -> anyhow::Result<()> {
    let response = resolve(url, None).await?;

    assert_eq!(response.status(), status);

    Ok(assert!(response
        .into_body()
        .data()
        .await
        .map(Result::unwrap)
        .is_some_and(|value| {
            assert_eq!(value.as_ref(), content);
            true
        })))
}

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
    #[case] url: &str,
    #[case] status: StatusCode,
    #[case] sled_id: &[u8],
) -> anyhow::Result<()> {
    let response = resolve(url, None).await?;

    assert_eq!(response.status(), status);

    Ok(assert!(response
        .into_body()
        .data()
        .await
        .map(Result::unwrap)
        .is_some_and(|value| {
            value.eq_ignore_ascii_case(sled_id)
        })))
}

// </editor-fold desc="// Tests ...">
