//! ## Utilities
//!

// Standard Library Imports
use core::ops::Div;
use std::collections::HashMap;

// Third-Party Imports
use axum::http::StatusCode;
use futures::prelude::*;
use serde_json::Value;

/// TODO
#[tracing::instrument(ret)]
pub async fn fetch_pokemon_weight(pokedex_id: u16) -> anyhow::Result<u32, (StatusCode, String)> {
    reqwest::get(format!("https://pokeapi.co/api/v2/pokemon/{pokedex_id}"))
        .map_err(|error| (StatusCode::SERVICE_UNAVAILABLE, error.to_string()))
        .and_then(|response: reqwest::Response| async move {
            if (199u16..300u16).contains(&response.status().as_u16()) {
                response
                    .json::<HashMap<String, Value>>()
                    .await
                    .map_err(|error| (StatusCode::EXPECTATION_FAILED, error.to_string()))
            } else {
                Err((response.status(), format!("{response:?}")))
            }
        })
        .await
        .and_then(|mut data: HashMap<String, Value>| {
            data.remove("weight").ok_or((
                StatusCode::UNPROCESSABLE_ENTITY,
                format!(
                    r#"missing "weight" key from: {}"#,
                    serde_json::to_string(&data).unwrap()
                ),
            ))
        })
        .and_then(|value: Value| value.as_u64().ok_or((StatusCode::NOT_FOUND, String::new())))
        .and_then(|value| {
            u32::try_from(value).map_err(|error| {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!("cannot downcast {value} to u32: {error}"),
                )
            })
        })
        .map(|value| value.div(10u32))
}