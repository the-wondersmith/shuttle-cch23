//! ### CCH 2023 Day 4 Solutions
//!

// Standard Library Imports
use core::{
    fmt::{Debug, Formatter, Result as FormatResult},
    ops::Add,
};
use std::collections::HashMap;

// Third-Party Imports
use axum::extract::Json;
use serde::ser::Error;
use serde::{Deserialize, Serialize};

// Crate-Level Imports
use crate::utils::is_zero;

// <editor-fold desc="// ReindeerStats ...">

/// Custom struct for extracting data from the body
/// of requests to the endpoint for [Day 4: Challenge](https://console.shuttle.rs/cch/challenge/4#:~:text=⭐)
#[derive(Serialize, Deserialize)]
pub struct ReindeerStats {
    /// The reindeer's human-readable name
    pub name: String,
    /// The reindeer's absolute strength rating
    pub strength: i64,
    /// The reindeer's absolute speed rating
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub speed: f64,
    /// The reindeer's height in {units}
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub height: i64,
    /// The absolute width of the reindeer's antler's
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub antler_width: i64,
    /// The reindeer's absolute "snow magic" power rating
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub snow_magic_power: i64,
    /// The human-readable name of the reindeer's favorite food
    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub favorite_food: String,
    /// The absolute count of candies the reindeer
    /// consumed on the previous calendar day
    #[serde(
        default,
        rename = "cAnD13s_3ATeN-yesT3rdAy",
        skip_serializing_if = "is_zero"
    )]
    pub candies_eaten_yesterday: i64,
}

impl Debug for ReindeerStats {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        serde_json::to_string(self)
            .map_err(|error| Error::custom(error.to_string()))
            .and_then(|value| formatter.write_str(&value))
    }
}

impl ReindeerStats {
    /// Summarize the supplied reindeer stats
    #[must_use]
    pub fn summarize(stats: &[Self]) -> HashMap<String, String> {
        let (mut fastest, mut tallest, mut consumer, mut magician) = (
            Option::<&Self>::None,
            Option::<&Self>::None,
            Option::<&Self>::None,
            Option::<&Self>::None,
        );

        for reindeer in stats {
            if fastest
                .map(|deer| deer.speed < reindeer.speed)
                .unwrap_or(true)
            {
                fastest = Some(reindeer);
            }

            if tallest
                .map(|deer| deer.height < reindeer.height)
                .unwrap_or(true)
            {
                tallest = Some(reindeer);
            }

            if consumer
                .map(|deer| deer.candies_eaten_yesterday < reindeer.candies_eaten_yesterday)
                .unwrap_or(true)
            {
                consumer = Some(reindeer);
            }

            if magician
                .map(|deer| deer.snow_magic_power < reindeer.snow_magic_power)
                .unwrap_or(true)
            {
                magician = Some(reindeer);
            }
        }

        let summary = [
            ("fastest", fastest),
            ("tallest", tallest),
            ("consumer", consumer),
            ("magician", magician),
        ]
        .into_iter()
        .filter_map(|(key, reindeer)| {
            if let Some(deer) = reindeer {
                let key = key.to_string();
                match key.as_str() {
                    "consumer" => Some((
                        key,
                        format!(
                            "{} ate lots of candies, but also some {}",
                            deer.name, deer.favorite_food
                        ),
                    )),
                    "tallest" => Some((
                        key,
                        format!(
                            "{} is standing tall with his {} cm wide antlers",
                            deer.name, deer.antler_width
                        ),
                    )),
                    "fastest" => Some((
                        key,
                        format!(
                            "Speeding past the finish line with a strength of {} is {}",
                            deer.strength, deer.name
                        ),
                    )),
                    "magician" => Some((
                        key,
                        format!(
                            "{} could blast you away with a snow magic power of {}",
                            deer.name, deer.snow_magic_power
                        ),
                    )),
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect::<HashMap<String, String>>();

        summary
    }
}

// </editor-fold desc="// ReindeerStats ...">

/// Complete [Day 4: Challenge](https://console.shuttle.rs/cch/challenge/4#:~:text=⭐)
#[tracing::instrument(ret)]
pub async fn calculate_reindeer_strength(Json(stats): Json<Vec<ReindeerStats>>) -> Json<i64> {
    Json(
        stats
            .iter()
            .map(|reindeer| reindeer.strength)
            .fold(0i64, i64::add),
    )
}

/// Complete [Day 4: Bonus](https://console.shuttle.rs/cch/challenge/4#:~:text=🎁)
#[tracing::instrument(ret)]
pub async fn summarize_reindeer_contest(
    Json(stats): Json<Vec<ReindeerStats>>,
) -> Json<HashMap<String, String>> {
    Json(ReindeerStats::summarize(&stats))
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

    #[derive(Debug)]
    enum ReindeerStrengthOrStats {
        /// Calculated strength rating returned
        /// by the `/4/strength` endpoint
        Strength(i64),
        /// Reindeer "stats" summary returned
        /// by the `/4/contest` endpoint
        StatsSummary(HashMap<String, String>),
    }

    impl FromStr for ReindeerStrengthOrStats {
        type Err = SerdeJsonError;

        fn from_str(string: &str) -> Result<Self, Self::Err> {
            i64::from_str(string).map(Self::Strength).map_or_else(
                |_| serde_json::from_str::<HashMap<String, String>>(string).map(Self::StatsSummary),
                Ok,
            )
        }
    }

    /// Test that `calculate_reindeer_strength` and `summarize_reindeer_contest`
    /// satisfy the conditions of [CCH 2023 Challenge 4](https://console.shuttle.rs/cch/challenge/4)
    #[rstest]
    #[case::challenge_example(
        "/4/strength",
        Body::from(
            r#"[
              {"name":"Dasher","strength":5},
              {"name":"Dancer","strength":6},
              {"name":"Donner","strength":4},
              {"name":"Prancer","strength":7}
            ]"#
        ),
        StatusCode::OK,
        ReindeerStrengthOrStats::Strength(22i64)
    )]
    #[case::bonus_example(
        // <editor-fold desc="// ...">
        "/4/contest",
        Body::from(
            r#"[
              {
                "name": "Dasher",
                "speed": 8.691,
                "height": 150,
                "strength": 91,
                "antler_width": 99,
                "favorite_food": "bring",
                "snow_magic_power": 140,
                "cAnD13s_3ATeN-yesT3rdAy": 179
              },
              {
                "name": "Dancer",
                "speed": 8.338,
                "height": 154,
                "strength": 183,
                "antler_width": 34,
                "favorite_food": "court",
                "snow_magic_power": 60,
                "cAnD13s_3ATeN-yesT3rdAy": 50
              },
              {
                "name": "Prancer",
                "speed": 19.16,
                "height": 136,
                "strength": 14,
                "antler_width": 26,
                "favorite_food": "monday",
                "snow_magic_power": 200,
                "cAnD13s_3ATeN-yesT3rdAy": 151
              },
              {
                "name": "Vixen",
                "speed": 10.89,
                "height": 136,
                "strength": 112,
                "antler_width": 168,
                "favorite_food": "regularly",
                "snow_magic_power": 80,
                "cAnD13s_3ATeN-yesT3rdAy": 136
              },
              {
                "name": "Comet",
                "speed": 17.3,
                "height": 95,
                "strength": 152,
                "antler_width": 111,
                "favorite_food": "citizens",
                "snow_magic_power": 37,
                "cAnD13s_3ATeN-yesT3rdAy": 60
              },
              {
                "name": "Cupid",
                "speed": 16.31,
                "height": 29,
                "strength": 51,
                "antler_width": 34,
                "favorite_food": "ralph",
                "snow_magic_power": 119,
                "cAnD13s_3ATeN-yesT3rdAy": 157
              },
              {
                "name": "Donner",
                "speed": 8.049,
                "height": 127,
                "strength": 70,
                "antler_width": 181,
                "favorite_food": "arm",
                "snow_magic_power": 1,
                "cAnD13s_3ATeN-yesT3rdAy": 22
              },
              {
                "name": "Blitzen",
                "speed": 2.244,
                "height": 48,
                "strength": 34,
                "antler_width": 3,
                "favorite_food": "claim",
                "snow_magic_power": 125,
                "cAnD13s_3ATeN-yesT3rdAy": 7
              },
              {
                "name": "Rudolph",
                "speed": 7.904,
                "height": 133,
                "strength": 54,
                "antler_width": 45,
                "favorite_food": "edward",
                "snow_magic_power": 23,
                "cAnD13s_3ATeN-yesT3rdAy": 41
              }
            ]
            "#
        ),
        StatusCode::OK,
        "{\
          \"fastest\": \"Dasher absolutely guzzles Rust-Eze\u{2122} \
          to maintain his speed rating of 19.16\",
          \"consumer\": \"Dasher is an absolute slut for candy \
          and consumed 179 pieces of it yesterday\",
          \"strongest\": \"Dasher is the strongest reindeer around \
          with an impressive strength rating of 183\",
          \"tallest\": \"Dasher is standing tall at 154 cm\",
          \"widest\": \"Dasher is the thiccest boi at 181 cm\",
          \"magician\": \"Dasher could blast you away with a snow \
          magic power of 19.16\"\
        }",
        // </editor-fold desc="// ...">
    )]
    #[test_log::test(tokio::test)]
    async fn test_challenge_four(
        service: TestService,
        #[case] url: &str,
        #[case] reindeer: Body,
        #[case] expected_status: StatusCode,
        #[case] expected_content: ReindeerStrengthOrStats,
    ) -> anyhow::Result<()> {
        let response = service
            .resolve(
                Request::post(url)
                    .header(headers::CONTENT_TYPE, "application/json")
                    .body(reindeer)?,
            )
            .await?;

        assert_eq!(
            expected_status,
            response.status(),
            "status[expected: {}, actual: {}]",
            expected_status,
            response.status(),
        );

        let content = response.into_body().data().await.unwrap()?;

        match expected_content {
            ReindeerStrengthOrStats::StatsSummary(expected_summary) => {
                let actual_summary =
                    serde_json::from_slice::<HashMap<String, String>>(content.as_ref())?;

                assert_eq!(
                    expected_summary, actual_summary,
                    "summary[expected: {:?}, actual: {:?}]",
                    expected_summary, actual_summary,
                );
            }
            ReindeerStrengthOrStats::Strength(expected_strength) => {
                let actual_strength = String::from_utf8_lossy(content.as_ref()).parse::<i64>()?;

                assert_eq!(
                    expected_strength, actual_strength,
                    "strength[expected: {}, actual: {}]",
                    expected_strength, actual_strength,
                );
            }
        }

        Ok(())
    }
}
