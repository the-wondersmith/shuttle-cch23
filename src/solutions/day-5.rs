//! ### CCH 2023 Day 5 Solutions
//!

// Standard Library Imports
use core::cmp;

// Third-Party Imports
use axum::{
    extract::{Json, Query},
    http::StatusCode,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Pagination {
    #[serde(default)]
    offset: usize,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    split: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum NameList {
    Unsplit(Vec<String>),
    Split(Vec<Vec<String>>),
}

/// Complete [Day 5: Challenge](https://console.shuttle.rs/cch/challenge/5#:~:text=⭐)
#[tracing::instrument(
    ret,
    skip_all,
    fields(
      names = names.len(),
      offset = pagination.offset,
      limit = pagination.limit.unwrap_or(names.len()),
      split = pagination.split.unwrap_or(0),
    )
)]
pub async fn slice_the_loop(
    Query(pagination): Query<Pagination>,
    Json(names): Json<Vec<String>>,
) -> Result<Json<NameList>, StatusCode> {
    let limit = pagination.limit.unwrap_or(names.len());
    let (start, end) = (
        pagination.offset,
        cmp::min(names.len(), pagination.offset + limit),
    );

    if names.len() < start || names.len() < end {
        Err(StatusCode::UNPROCESSABLE_ENTITY)
    } else {
        let split = pagination.split.unwrap_or(0usize);
        let names = &names[start..end];

        Ok(Json(if 0 < split {
            NameList::Split(
                names
                    .iter()
                    .chunks(split)
                    .into_iter()
                    .map(|chunk| chunk.map(String::from).collect_vec())
                    .collect_vec(),
            )
        } else {
            NameList::Unsplit(names.to_vec())
        }))
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
}
