//! ### CCH 2023 Day 8 Solutions
//!

// Third-Party Imports
use axum::{
    extract::{Json, Path},
    http::StatusCode,
};

// Crate-Level Imports
use crate::utils;

/// Complete [Day 8: Challenge](https://console.shuttle.rs/cch/challenge/8#:~:text=⭐)
#[tracing::instrument(ret)]
pub async fn fetch_pokemon_weight(
    Path(pokedex_id): Path<u16>,
) -> Result<Json<f64>, (StatusCode, String)> {
    Ok(Json(utils::fetch_pokemon_weight(pokedex_id).await?))
}

/// Complete [Day 8: Bonus](https://console.shuttle.rs/cch/challenge/8#:~:text=🎁)
#[allow(non_upper_case_globals)]
#[tracing::instrument(ret)]
pub async fn calculate_pokemon_impact_momentum(
    Path(pokedex_id): Path<u16>,
) -> Result<Json<f64>, (StatusCode, String)> {
    /// Gravitational acceleration in m/s²
    const gravity: f64 = 9.825;
    /// Chimney height in meters
    const drop_height: f64 = 10.0;

    let poke_weight = utils::fetch_pokemon_weight(pokedex_id).await?;

    // Calculate the final speed with kinematic equation
    let final_speed = (2.0 * gravity * drop_height).sqrt();

    // Calculate the impact momentum
    let momentum = poke_weight * final_speed;

    Ok(Json(momentum))
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

    // <editor-fold desc="// Helper Types ...">

    #[derive(Debug, PartialEq)]
    enum PokemonWeightOrImpactMomentum {
        /// Weight-in-kilograms value returned by
        /// the `/8/weight/:pokedex_id` endpoint
        Weight(u32),
        /// Momentum-in-newton-seconds value returned
        /// by the `/8/drop/:pokedex_id` endpoint
        ImpactMomentum(f64),
    }

    impl FromStr for PokemonWeightOrImpactMomentum {
        type Err = anyhow::Error;

        fn from_str(string: &str) -> Result<Self, Self::Err> {
            u32::from_str(string).map(Self::Weight).map_or_else(
                |_| {
                    f64::from_str(string).map(Self::ImpactMomentum).map_or_else(
                        |_| Err(Self::Err::msg(format!("unparsable value: {string}"))),
                        Ok,
                    )
                },
                Ok,
            )
        }
    }

    impl PartialEq<u32> for PokemonWeightOrImpactMomentum {
        fn eq(&self, other: &u32) -> bool {
            match self {
                Self::Weight(weight) => weight.eq(other),
                _ => false,
            }
        }
    }

    impl PartialEq<f64> for PokemonWeightOrImpactMomentum {
        fn eq(&self, other: &f64) -> bool {
            match self {
                Self::ImpactMomentum(momentum) => momentum.eq(other),
                _ => false,
            }
        }
    }

    // </editor-fold desc="// Helper Types ...">

    /// Test that `fetch_pokemon_weight` and `calculate_pokemon_impact_momentum`
    /// satisfy the conditions of [CCH 2023 Challenge 6](https://console.shuttle.rs/cch/challenge/8)
    #[rstest]
    #[case::challenge_example("/8/weight/25", StatusCode::OK, "6")]
    #[case::bonus_example("/8/drop/25", StatusCode::OK, "84.10707461325713")]
    #[test_log::test(tokio::test)]
    async fn test_challenge_eight(
        service: TestService,
        #[case] url: &str,
        #[case] expected_status: StatusCode,
        #[case] expected_value: PokemonWeightOrImpactMomentum,
    ) -> anyhow::Result<()> {
        let response = service.resolve(url).await?;

        assert_eq!(
            expected_status,
            response.status(),
            "status[expected: {}, actual: {}]",
            expected_status,
            response.status(),
        );

        let actual_value = response
            .into_body()
            .data()
            .await
            .unwrap()
            .map_err(|error| anyhow::Error::msg(error.to_string()))
            .and_then(|content| {
                String::from_utf8_lossy(content.as_ref()).parse::<PokemonWeightOrImpactMomentum>()
            })?;

        assert_eq!(
            expected_value, actual_value,
            "value[expected: {:?}, actual: {:?}]",
            expected_value, actual_value
        );

        Ok(())
    }
}
