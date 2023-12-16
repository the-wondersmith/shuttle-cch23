//! ## Solutions
//!

// Module Declarations
#[path = "day-1.rs"]
pub mod day_1;
#[path = "day-11.rs"]
pub mod day_11;
#[path = "day-12.rs"]
pub mod day_12;
#[path = "day-13.rs"]
pub mod day_13;
#[path = "day-14.rs"]
pub mod day_14;
#[path = "day-15.rs"]
pub mod day_15;
#[path = "day-18.rs"]
pub mod day_18;
#[path = "day-19.rs"]
pub mod day_19;
#[path = "day-20.rs"]
pub mod day_20;
#[path = "day-21.rs"]
pub mod day_21;
#[path = "day-22.rs"]
pub mod day_22;
#[path = "day-4.rs"]
pub mod day_4;
#[path = "day-5.rs"]
pub mod day_5;
#[path = "day-6.rs"]
pub mod day_6;
#[path = "day-7.rs"]
pub mod day_7;
#[path = "day-8.rs"]
pub mod day_8;

#[allow(unused_imports)]
pub use self::{
    day_1::{calculate_sled_id, cube_the_bits},
    day_11::{calculate_magical_red_pixel_count, serve_static_asset},
    day_12::{
        analyze_ulids, retrieve_packet_id_timestamp, santas_ulid_hug_box, store_packet_id_timestamp,
    },
    day_13::{
        create_orders, most_popular_gift, reset_db_schema, simple_sql_select, total_order_count,
    },
    day_14::{render_html_safe, render_html_unsafe},
    day_15::{assess_naughty_or_nice, game_of_the_year},
    day_4::{calculate_reindeer_strength, summarize_reindeer_contest},
    day_6::count_elves,
    day_7::{bake_cookies_from_recipe_and_pantry, decode_cookie_recipe},
    day_8::{calculate_pokemon_impact_momentum, fetch_pokemon_weight},
    day_minus_1::{hello_world, throw_error},
};
pub mod day_minus_1 {
    use axum::{http::StatusCode, response::IntoResponse};

    /// Complete [Day -1: Challenge](https://console.shuttle.rs/cch/challenge/-1#:~:text=⭐)
    #[tracing::instrument(ret)]
    pub async fn hello_world() -> &'static str {
        "Hello Shuttle CCH 2023!"
    }

    /// Complete [Day -1: Bonus](https://console.shuttle.rs/cch/challenge/-1#:~:text=🎁)
    #[tracing::instrument(ret)]
    pub async fn throw_error() -> impl IntoResponse {
        (StatusCode::INTERNAL_SERVER_ERROR, "Gimme them bonus points")
    }
}
