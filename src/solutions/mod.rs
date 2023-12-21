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
        create_orders, most_popular_gift, reset_day_13_schema, simple_sql_select, total_order_count,
    },
    day_14::{render_html_safe, render_html_unsafe},
    day_15::{assess_naughty_or_nice, game_of_the_year},
    day_18::{
        create_regions, get_order_count_by_region, get_top_n_gifts_by_region, reset_day_18_schema,
    },
    day_19::{
        connect_to_chat_room, get_current_chat_count, play_socket_ping_pong, reset_chat_count,
        ChatRoomState,
    },
    day_20::{get_archived_file_count, get_total_archived_file_size, git_blame_cookie_hunt},
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

        /// Test that `hello_world` and `throw_error`
        /// satisfy the conditions of [CCH 2023 Challenge -1](https://console.shuttle.rs/cch/challenge/-1)
        #[rstest]
        #[case::hello_world(
            "/",
            StatusCode::OK,  // no-reformat
            b"Hello Shuttle CCH 2023!",
        )]
        #[case::throw_error(
            "/-1/error",
            StatusCode::INTERNAL_SERVER_ERROR,
            b"Gimme them bonus points"
        )]
        #[test_log::test(tokio::test)]
        async fn test_challenge_minus_one(
            service: TestService,
            #[case] url: &str,
            #[case] expected_status: StatusCode,
            #[case] expected_content: &[u8],
        ) -> anyhow::Result<()> {
            let response = service.resolve(url).await?;

            assert_eq!(
                expected_status,
                response.status(),
                "status[expected: {}, actual: {}]",
                expected_status,
                response.status(),
            );

            let content = response.into_body().data().await.unwrap()?;

            Ok(assert!(
                content.eq_ignore_ascii_case(expected_content),
                "content[expected: {}, actual: {}]",
                String::from_utf8_lossy(expected_content),
                String::from_utf8_lossy(content.as_ref()),
            ))
        }
    }
}
