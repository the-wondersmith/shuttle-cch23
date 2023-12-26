#![allow(unused_imports)]
//! ### CCH 2023 Day 22 Solutions
//!

// Standard Library Imports
use core::{
    cmp,
    marker::PhantomData,
    ops::{Add, AddAssign, BitXor, Div, Mul, Sub},
};
use std::{collections::VecDeque, str::FromStr};

// Third-Party Imports
use axum::{
    extract::{FromRef, FromRequest},
    http::StatusCode,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

// <editor-fold desc="// Portal ...">

type Portal = (usize, usize);

// </editor-fold desc="// Portal ...">

// <editor-fold desc="// Star ...">

#[derive(Eq, Ord, Copy, Hash, Clone, Debug, PartialEq, PartialOrd)]
pub struct Star(i32, i32, i32);

impl FromStr for Star {
    type Err = (StatusCode, String);

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let points = line
            .split_whitespace()
            .filter_map(|point| match point.parse::<i32>() {
                Ok(point) => Some(point),
                Err(error) => {
                    tracing::error!("couldn't parse int from value: {point} -> {error:?}");
                    None
                }
            })
            .collect_vec();

        if points.len() == 3 {
            Ok(Self(points[0], points[1], points[2]))
        } else {
            Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("expected a three-tuple of i32's, but got: {line:?} -> {points:?}"),
            ))
        }
    }
}

impl Star {
    pub fn distance(&self, other: &Self) -> f64 {
        let delta_x = self.0 - other.0;
        let delta_y = self.1 - other.1;
        let delta_z = self.2 - other.2;

        ((delta_x * delta_x + delta_y * delta_y + delta_z * delta_z) as f64).sqrt()
    }
}

// </editor-fold desc="// Star ...">

// <editor-fold desc="// StarPortalChart ...">

#[derive(Clone, Debug)]
pub struct StarPortalChart {
    stars: Vec<Star>,
    portals: Vec<Portal>,
}

impl FromStr for StarPortalChart {
    type Err = (StatusCode, String);

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut lines = text.split('\n');

        let star_count = lines
            .next()
            .ok_or((
                StatusCode::UNPROCESSABLE_ENTITY,
                "missing initial star count".to_string(),
            ))?
            .parse::<usize>()
            .map_err(|error| (StatusCode::UNPROCESSABLE_ENTITY, error.to_string()))?;

        let stars = (&mut lines)
            .take(star_count)
            .flat_map(|line| match line.parse::<Star>() {
                Ok(star) => Some(star),
                Err(_) => {
                    tracing::error!("error parsing line: {line}");
                    None
                }
            })
            .collect_vec();

        if stars.len() != star_count {
            return Err((
                StatusCode::EXPECTATION_FAILED,
                format!("expected {star_count} stars, got {}", stars.len()),
            ));
        }

        let portal_count = lines
            .next()
            .ok_or((
                StatusCode::UNPROCESSABLE_ENTITY,
                "missing portal count".to_string(),
            ))?
            .parse::<usize>()
            .map_err(|error| (StatusCode::EXPECTATION_FAILED, error.to_string()))?;

        let portals = lines
            .flat_map(|line| {
                let ids = line
                    .split_whitespace()
                    .flat_map(|id| id.parse::<usize>().ok())
                    .collect_vec();

                if ids.len() == 2 {
                    Some((ids[0], ids[1]))
                } else {
                    None
                }
            })
            .collect_vec();

        if portals.len() == portal_count {
            Ok(Self { stars, portals })
        } else {
            Err((
                StatusCode::EXPECTATION_FAILED,
                format!("expected {portal_count} portals, got {}", portals.len()),
            ))
        }
    }
}

impl StarPortalChart {
    fn shortest_path(&self) -> Result<Vec<Star>, (StatusCode, String)> {
        if self.stars.is_empty() || self.portals.is_empty() {
            return Err((
                StatusCode::EXPECTATION_FAILED,
                String::from("no stars or portals provided"),
            ));
        }

        let (start, end) = (0usize, self.stars.len() - 1);

        let mut visited = vec![false; end + 1];
        let mut routes = vec![vec![]; end + 1];
        let mut unexplored = VecDeque::new();

        visited[start] = true;
        routes[start].push(start);
        unexplored.push_back(start);

        while let Some(current) = unexplored.pop_front() {
            for portal in &self.portals {
                let (origin, destination) = *portal;

                if origin == current && !visited[destination] {
                    visited[destination] = true;

                    let mut route_b = routes[origin].clone();

                    route_b.push(destination);
                    routes[destination] = route_b;
                    unexplored.push_back(destination);
                }
            }
        }

        if !visited[end] {
            Err((StatusCode::NOT_FOUND, "".to_string()))
        } else {
            Ok(routes[end].iter().map(|idx| self.stars[*idx]).collect_vec())
        }
    }
}

// </editor-fold desc="// StarPortalChart ...">

/// Complete [Day 22: Task](https://console.shuttle.rs/cch/challenge/22#:~:text=⭐️)
#[tracing::instrument(skip_all, fields(int.count, loner))]
pub async fn locate_lonely_int(text: String) -> Result<String, (StatusCode, String)> {
    let mut ints = 0u64;

    let loner: usize = text
        .split_whitespace()
        .inspect(|_| ints.add_assign(&1u64))
        .filter_map(|line| line.parse::<u64>().ok())
        .fold(0u64, BitXor::bitxor)
        .try_into()
        .map_err(|error| (StatusCode::UNPROCESSABLE_ENTITY, format!("{error:?}")))?;

    tracing::Span::current().record("int.count", ints);
    tracing::Span::current().record("loner", loner);

    tracing::info!("gift id located");

    Ok("🎁".repeat(loner))
}

/// Complete [Day 22: Task](https://console.shuttle.rs/cch/challenge/22#:~:text=⭐️)
#[tracing::instrument(ret, skip_all, fields(stars, portals, distance))]
pub async fn analyze_star_chart(text: String) -> Result<String, (StatusCode, String)> {
    let chart = text.parse::<StarPortalChart>()?;

    tracing::Span::current().record("stars", chart.stars.len());
    tracing::Span::current().record("portals", chart.portals.len());

    let path = chart.shortest_path()?;

    let real_distance = path
        .iter()
        .tuple_windows::<(&Star, &Star)>()
        .fold(0.0f64, |distance, (origin, destination)| {
            distance + origin.distance(destination)
        });

    tracing::Span::current().record("distance", real_distance);

    Ok(format!("{} {:.3}", path.len() - 1, real_distance))
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
