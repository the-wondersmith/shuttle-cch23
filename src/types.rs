//! ## Custom Types
//!

use axum::{
    async_trait,
    extract::{rejection::PathRejection, FromRequestParts, Path},
    http::request::Parts,
};
use serde_json::value::Value;

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
                if let Ok(value) = serde_json::from_str::<Value>(part) {
                    value
                } else {
                    Value::from(part)
                }
            })
            .collect::<Vec<Value>>();

        Ok(Self(values))
    }
}
