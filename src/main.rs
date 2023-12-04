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
#[cfg(test)]
mod tests;

pub mod types;

/// Run the project
#[tracing::instrument]
#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    Ok(router().into())
}

/// Create the project's main `Router` instance
#[tracing::instrument]
pub fn router() -> Router {
    Router::new()
        .route("/", routing::get(hello_world))
        .route("/-1/error", routing::get(throw_error))
        .route("/1/*packets", routing::get(calculate_sled_id))
}

/// Complete [Challenge -1: Task](https://console.shuttle.rs/cch/challenge/-1#:~:text=one%20that%20counts.-,%E2%AD%90,-Task%201%3A%20Everything)
#[tracing::instrument(ret)]
pub async fn hello_world() -> &'static str {
    "Hello Shuttle CCH 2023!"
}

/// Complete [Challenge -1: Bonus](https://console.shuttle.rs/cch/challenge/-1#:~:text=the%20bonus%20task.-,%F0%9F%8E%81,-Task%202%3A%20Fake)
#[tracing::instrument(ret)]
pub async fn throw_error() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Gimme them bonus points")
}

/// Complete [Challenge 1: Task](https://console.shuttle.rs/cch/challenge/1#:~:text=to%20restore%20order.-,%E2%AD%90,-Task%201%3A%20Cube)
#[allow(dead_code)]
#[tracing::instrument(ret)]
#[cfg_attr(tarpaulin, coverage(off))]
#[cfg_attr(tarpaulin, tarpaulin::skip)]
pub async fn cube_the_bits(Path(values): Path<(u32, u32)>) -> impl IntoResponse {
    (
        StatusCode::OK,
        std::ops::BitXor::bitxor(values.0, values.1)
            .pow(3u32)
            .to_string(),
    )
}

/// Complete [Challenge 1: Bonus](https://console.shuttle.rs/cch/challenge/1#:~:text=1728-,%F0%9F%8E%81,-Task%202%3A%20The)
#[tracing::instrument(ret)]
pub async fn calculate_sled_id(
    types::VariadicPathValues(packets): types::VariadicPathValues,
) -> impl IntoResponse {
    // let mut bad_packets: Vec<Value> = Vec::new();

    let sled_id = packets
        .into_iter()
        .filter_map(|value| {
            if let Value::Number(val) = value {
                val.as_i64()
            } else {
                // bad_packets.push(value);
                None
            }
        })
        .fold(0i64, std::ops::BitXor::bitxor)
        .pow(3u32);

    // if bad_packets.is_empty() {
    //     (StatusCode::OK, sled_id.to_string())
    // } else {
    //     (
    //         StatusCode::BAD_REQUEST,
    //         format!("Non-numeric packet ids: {bad_packets:?}"),
    //     )
    // }

    (StatusCode::OK, sled_id.to_string())
}
