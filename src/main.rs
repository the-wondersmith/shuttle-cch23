#![forbid(unsafe_code)]
#![cfg_attr(tarpaulin, feature(register_tool))]
#![cfg_attr(tarpaulin, register_tool(tarpaulin))]
#![cfg_attr(tarpaulin, feature(coverage_attribute))]
#![deny(missing_docs, missing_debug_implementations)]

//! # [`shuttle.rs`](https://shuttle.rs/) Christmas Code Hunt 2023
//!

// Third-Party Imports
use axum::{routing::get, Router};

/// TODO(the-wondersmith): DOCUMENTATION
#[tracing::instrument]
async fn hello_world() -> &'static str {
    "Hello, world!"
}

/// TODO(the-wondersmith): DOCUMENTATION
#[tracing::instrument]
#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route("/", get(hello_world));

    Ok(router.into())
}
