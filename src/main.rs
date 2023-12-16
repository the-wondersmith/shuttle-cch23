#![forbid(unsafe_code)]
#![feature(entry_insert, const_trait_impl)]
#![cfg_attr(tarpaulin, feature(register_tool))]
#![cfg_attr(tarpaulin, register_tool(tarpaulin))]
#![cfg_attr(tarpaulin, feature(coverage_attribute))]
#![deny(missing_debug_implementations)]

//! # [`shuttle.rs`](https://shuttle.rs/) Christmas Code Hunt 2023
//!

// Module Declarations
pub mod solutions;
#[cfg(test)]
mod tests;
pub mod types;
pub mod utils;

// Third-Party Imports
use axum::routing::{self, Router as AxumRouter};
use shuttle_axum::ShuttleAxum as ShuttleAxumApp;
use shuttle_persist::{Persist, PersistInstance as Persistence};
use shuttle_secrets::{SecretStore, Secrets};
use shuttle_shared_db::Postgres as PgDb;

// Crate-Level Imports
use crate::types::ShuttleAppState;

/// Run the project
#[cfg_attr(tarpaulin, coverage(off))]
#[cfg_attr(tarpaulin, tarpaulin::skip)]
#[tracing::instrument(skip_all)]
#[shuttle_runtime::main]
async fn main(
    #[PgDb] pool: sqlx::PgPool,
    #[Secrets] secrets: SecretStore,
    #[Persist] persistence: Persistence,
) -> ShuttleAxumApp {
    let state = ShuttleAppState::initialize(pool, Some(secrets), None, Some(persistence))?;

    Ok(router(state).into())
}

/// Create the project's main `Router` instance
#[tracing::instrument(skip(state))]
pub fn router(state: ShuttleAppState) -> AxumRouter {
    routing::Router::new()
        .route("/", routing::get(solutions::hello_world))
        .route("/-1/error", routing::get(solutions::throw_error))
        .route("/1/*packets", routing::get(solutions::calculate_sled_id))
        .route(
            "/4/contest",
            routing::post(solutions::summarize_reindeer_contest),
        )
        .route(
            "/4/strength",
            routing::post(solutions::calculate_reindeer_strength),
        )
        .route("/6", routing::post(solutions::count_elves))
        .route(
            "/7/bake",
            routing::get(solutions::bake_cookies_from_recipe_and_pantry)
                .post(solutions::bake_cookies_from_recipe_and_pantry),
        )
        .route(
            "/7/decode",
            routing::get(solutions::decode_cookie_recipe).post(solutions::decode_cookie_recipe),
        )
        .route(
            "/8/weight/:pokedex_id",
            routing::get(solutions::fetch_pokemon_weight),
        )
        .route(
            "/8/drop/:pokedex_id",
            routing::get(solutions::calculate_pokemon_impact_momentum),
        )
        .route(
            "/11/assets/:asset",
            routing::get(solutions::serve_static_asset),
        )
        .route(
            "/11/red_pixels",
            routing::post(solutions::calculate_magical_red_pixel_count),
        )
        .route(
            "/12/save/:packet_it",
            routing::post(solutions::store_packet_id_timestamp),
        )
        .route(
            "/12/load/:packet_it",
            routing::get(solutions::retrieve_packet_id_timestamp),
        )
        .route("/12/ulids", routing::post(solutions::santas_ulid_hug_box))
        .route(
            "/12/ulids/:weekday",
            routing::post(solutions::analyze_ulids),
        )
        .route("/13/sql", routing::get(solutions::simple_sql_select))
        .route("/13/reset", routing::post(solutions::reset_db_schema))
        .route("/13/orders", routing::post(solutions::create_orders))
        .route(
            "/13/orders/total",
            routing::get(solutions::total_order_count),
        )
        .route(
            "/13/orders/popular",
            routing::get(solutions::most_popular_gift),
        )
        .route("/14/safe", routing::post(solutions::render_html_safe))
        .route("/14/unsafe", routing::post(solutions::render_html_unsafe))
        .route("/15/nice", routing::post(solutions::assess_naughty_or_nice))
        .route("/15/game", routing::post(solutions::game_of_the_year))
        .with_state(state)
}
