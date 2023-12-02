#![forbid(unsafe_code)]
#![cfg_attr(tarpaulin, feature(register_tool))]
#![cfg_attr(tarpaulin, register_tool(tarpaulin))]
#![cfg_attr(tarpaulin, feature(coverage_attribute))]
#![deny(missing_docs, missing_debug_implementations)]

//! # [`shuttle.rs`](https://shuttle.rs/) Christmas Code Hunt 2023
//!

// Third-Party Imports
use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing, Router};
use serde_json::Value;

// Module Declarations
pub mod types;

/// TODO(the-wondersmith): DOCUMENTATION
#[tracing::instrument]
#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", routing::get(hello_world))
        .route("/-1/error", routing::get(throw_error))
        .route("/1/*packets", routing::get(calculate_sled_id));

    Ok(router.into())
}

/// Complete [Challenge -1: Task](https://console.shuttle.rs/cch/challenge/-1#:~:text=one%20that%20counts.-,%E2%AD%90,-Task%201%3A%20Everything)
#[tracing::instrument]
async fn hello_world() -> &'static str {
    "Hello, world!"
}

/// Complete [Challenge -1: Bonus](https://console.shuttle.rs/cch/challenge/-1#:~:text=the%20bonus%20task.-,%F0%9F%8E%81,-Task%202%3A%20Fake)
#[tracing::instrument]
async fn throw_error() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Gimme them bonus points")
}

/// Complete [Challenge 1: Task](https://console.shuttle.rs/cch/challenge/1#:~:text=to%20restore%20order.-,%E2%AD%90,-Task%201%3A%20Cube)
#[allow(dead_code)]
#[tracing::instrument]
async fn cube_the_bits(Path(values): Path<(u32, u32)>) -> impl IntoResponse {
    (
        StatusCode::OK,
        std::ops::BitXor::bitxor(values.0, values.1)
            .pow(3u32)
            .to_string(),
    )
}

/// Complete [Challenge 1: Bonus](https://console.shuttle.rs/cch/challenge/1#:~:text=1728-,%F0%9F%8E%81,-Task%202%3A%20The)
#[tracing::instrument]
async fn calculate_sled_id(
    types::VariadicPathValues(packets): types::VariadicPathValues,
) -> impl IntoResponse {
    tracing::info!(
        "Calculating sled id from packet ids: {}",
        serde_json::to_string(&packets).unwrap()
    );

    let sled_id = packets
        .iter()
        .filter_map(|value| {
            if let Value::Number(val) = value {
                val.as_u64()
            } else {
                tracing::warn!("Non-numeric packet id: {value:?}");
                None
            }
        })
        .fold(0u64, std::ops::BitXor::bitxor)
        .pow(3u32);

    (StatusCode::OK, sled_id.to_string())
}
