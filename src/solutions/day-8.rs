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
