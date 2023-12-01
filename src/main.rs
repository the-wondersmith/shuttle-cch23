#![forbid(unsafe_code)]
#![cfg_attr(tarpaulin, feature(register_tool))]
#![cfg_attr(tarpaulin, register_tool(tarpaulin))]
#![cfg_attr(tarpaulin, feature(coverage_attribute))]
#![deny(missing_docs, missing_debug_implementations)]

//! # [`shuttle.rs`](https://shuttle.rs/) Christmas Code Hunt 2023
//!

// Third-Party Imports
use axum::{http::StatusCode, response::IntoResponse, routing, Router};

/// TODO(the-wondersmith): DOCUMENTATION
#[tracing::instrument]
#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", routing::get(hello_world))
        .route("/-1/error", routing::get(throw_error));

    Ok(router.into())
}

/// Complete [Challenge -1: Task 1](https://console.shuttle.rs/cch/challenge/-1#:~:text=one%20that%20counts.-,%E2%AD%90,-Task%201%3A%20Everything)
#[tracing::instrument]
async fn hello_world() -> &'static str {
    "Hello, world!"
}

/// Complete [Challenge -1: Task 2](https://console.shuttle.rs/cch/challenge/-1#:~:text=the%20bonus%20task.-,%F0%9F%8E%81,-Task%202%3A%20Fake)
#[tracing::instrument]
async fn throw_error() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Gimme them bonus points")
}
