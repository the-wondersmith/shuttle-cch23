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

use crate::types::PantryInventory;
use once_cell::sync::Lazy;
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
use rstest::{fixture, rstest};
use serde_json::{error::Error as SerdeJsonError, Value};
use shuttle_shared_db::Postgres as ShuttleDB;
use tower::{MakeService, ServiceExt};

// Crate-Level Imports
use super::{
    router,
    solutions::{calculate_sled_id, cube_the_bits, hello_world, throw_error},
    types,
};

// <editor-fold desc="// Types ...">

#[derive(Debug)]
enum RecipeOrBakeResult {
    /// Decoded cookie recipe returned
    /// by the `/7/decode` endpoint
    Recipe(types::CookieRecipe),
    /// "Baking" summary returned
    /// by the `/7/bake` endpoint
    BakeResult(types::CookieRecipeInventory),
}

#[derive(Debug)]
enum ReindeerStrengthOrStats {
    /// Calculated strength rating returned
    /// by the `/4/strength` endpoint
    Strength(i64),
    /// Reindeer "stats" summary returned
    /// by the `/4/contest` endpoint
    StatsSummary(HashMap<String, String>),
}

#[derive(Debug, PartialEq)]
enum PokemonWeightOrImpactMomentum {
    /// Weight-in-kilograms value returned by
    /// the `/8/weight/:pokedex_id` endpoint
    Weight(u32),
    /// Momentum-in-newton-seconds value returned
    /// by the `/8/drop/:pokedex_id` endpoint
    ImpactMomentum(f64),
}
impl FromStr for RecipeOrBakeResult {
    type Err = SerdeJsonError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        serde_json::from_str::<types::CookieRecipe>(string)
            .map(Self::Recipe)
            .map_or_else(
                |_| {
                    serde_json::from_str::<types::CookieRecipeInventory>(string)
                        .map(Self::BakeResult)
                },
                Ok,
            )
    }
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

// </editor-fold desc="// Types ...">

// <editor-fold desc="// Constants ...">

// </editor-fold desc="// Constants ...">

// <editor-fold desc="// Fixtures ...">

#[fixture]
fn service() -> TestService {
    TestService::default()
}

// </editor-fold desc="// Fixtures ...">

// <editor-fold desc="// Utilities ...">

trait IntoBody {
    fn into_body(self) -> Body;
}

trait IntoMethod {
    fn into_method(self) -> Method;
}
trait TryIntoRequest<T: Debug> {
    fn into_request(self) -> anyhow::Result<Request<T>>;
}

impl IntoBody for Body {
    fn into_body(self) -> Body {
        self
    }
}

impl<T: IntoBody> IntoBody for Option<T> {
    fn into_body(self) -> Body {
        self.map_or_else(Body::empty, IntoBody::into_body)
    }
}

impl IntoMethod for Method {
    fn into_method(self) -> Method {
        self
    }
}

impl<T: IntoMethod> IntoMethod for Option<T> {
    fn into_method(self) -> Method {
        self.map_or(Method::GET, IntoMethod::into_method)
    }
}

impl TryIntoRequest<Body> for &str {
    fn into_request(self) -> anyhow::Result<Request<Body>> {
        Ok(Request::get(self).body(Body::empty())?)
    }
}

impl<T: Debug> TryIntoRequest<T> for Request<T> {
    fn into_request(self) -> anyhow::Result<Self> {
        Ok(self)
    }
}

impl TryIntoRequest<Body> for Builder {
    fn into_request(self) -> anyhow::Result<Request<Body>> {
        Ok(self.body(Body::empty())?)
    }
}

impl<R: Default + Debug, T: TryIntoRequest<R>> TryIntoRequest<R> for Option<T> {
    fn into_request(self) -> anyhow::Result<Request<R>> {
        self.map_or_else(
            || Ok(Request::get("/").body(R::default())?),
            TryIntoRequest::into_request,
        )
    }
}

impl<U: AsRef<str>, B: IntoBody, M: IntoMethod> TryIntoRequest<Body> for (U, Option<B>, M) {
    fn into_request(self) -> anyhow::Result<Request<Body>> {
        let mut request = Request::builder()
            .uri(self.0.as_ref())
            .method(self.2.into_method());

        if self.1.is_some() {
            request = request.header("content-type", "application/json");
        }

        Ok(request.body(self.1.into_body())?)
    }
}

#[derive(Debug)]
struct TestService(Router);

impl Default for TestService {
    fn default() -> Self {
        // let db = ShuttleDB::new();
        // let state = types::ShuttleAppState::initialize(None, None, None);
        // Self(router(state))
        todo!()
    }
}

impl TestService {
    /// Bounce the supplied request body off the project's
    /// `axum::Router` at the specified path and return the
    /// resolved response
    pub async fn resolve<Resolvable: TryIntoRequest<Body>>(
        &self,
        request: Resolvable,
    ) -> anyhow::Result<Response<BoxBody>> {
        let request = request.into_request()?;

        Ok(self.0.clone().oneshot(request).await?)
    }
}

// </editor-fold desc="// Utilities ...">

// <editor-fold desc="// Tests ...">

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

/// Test that `calculate_sled_id`
/// satisfies the conditions of [CCH 2023 Challenge 1](https://console.shuttle.rs/cch/challenge/1)
#[rstest]
#[case::challenge_example("/1/4/8", StatusCode::OK, b"1728")]
#[case::bonus_example_one("/1/10", StatusCode::OK, b"1000")]
#[case::bonus_example_two("/1/4/5/8/10", StatusCode::OK, b"27")]
#[case::negative_packet_ids("/1/-12/45/-6", StatusCode::OK, b"42875")]
#[case::mixed_packet_ids("/1/95/7552/sixty-four", StatusCode::BAD_REQUEST, b"0")]
#[case::non_numeric_packet_ids("/1/fifty-five/fourteen", StatusCode::BAD_REQUEST, b"0")]
#[test_log::test(tokio::test)]
async fn test_challenge_one(
    service: TestService,
    #[case] url: &str,
    #[case] expected_status: StatusCode,
    #[case] expected_sled_id: &[u8],
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
        content
            .eq_ignore_ascii_case(expected_sled_id)
            .bitor(url.split('/').all(|part| {
                part.is_empty()
                    || part.parse::<i64>().is_ok()
                    || content
                        .as_ref()
                        .windows(part.len())
                        .any(|chunk| chunk == part.as_bytes())
            })),
        "content[expected: {}, actual: {}]",
        String::from_utf8_lossy(expected_sled_id),
        String::from_utf8_lossy(content.as_ref()),
    ))
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

/// TODO: [CCH 2023 Challenge 5](https://console.shuttle.rs/cch/challenge/5)
#[rstest]
#[ignore = "Grinch-ed™"]
#[test_log::test(tokio::test)]
async fn test_challenge_five() {
    todo!()
}

/// Test that `count_elves` satisfies the conditions of
/// [CCH 2023 Challenge 6](https://console.shuttle.rs/cch/challenge/6)
#[rstest]
#[case::challenge_example(
    "The mischievous elf peeked out from \
     behind the toy workshop, and another \
     elf joined in the festive dance. Look, \
     there is also an elf on that shelf! \
    ",
    StatusCode::OK,
    types::ElfShelfCountSummary {
        loose_elves: 4u64,
        bare_shelves: 1u64,
        shelved_elves: 0u64,
    },
)]
#[case::bonus_example1(
    "there is an elf on a shelf on an elf. \
     there is also another shelf in Belfast. \
    ",
    StatusCode::OK,
    types::ElfShelfCountSummary {
        loose_elves: 5u64,
        bare_shelves: 1u64,
        shelved_elves: 1u64,
    },
)]
#[case::bonus_example2(
    "elf elf elf",
    StatusCode::OK,
    types::ElfShelfCountSummary {
        loose_elves: 3u64,
        bare_shelves: 0u64,
        shelved_elves: 0u64,
    },
)]
#[case::bonus_example3(
    "In the quirky town of Elf stood an enchanting \
     shop named 'The Elf & Shelf.' Managed \
     by Wally, a mischievous elf with a knack \
     for crafting exquisite shelves, the \
     shop was a bustling hub of elf after \
     elf who wanted to see their dear elf \
     in Belfast. \
    ",
    StatusCode::OK,
    types::ElfShelfCountSummary {
        loose_elves: 6u64,
        bare_shelves: 0u64,
        shelved_elves: 0u64,
    },
)]
#[case::bonus_example4(
    "elf elf elf on a shelf",
    StatusCode::OK,
    types::ElfShelfCountSummary {
        loose_elves: 4u64,
        bare_shelves: 0u64,
        shelved_elves: 1u64,
    },
)]
#[case::bonus_example5(
    "In Belfast I heard an elf on a shelf \
     on a shelf on a \
    ",
    StatusCode::OK,
    types::ElfShelfCountSummary {
        loose_elves: 4u64,
        bare_shelves: 0u64,
        shelved_elves: 2u64,
    },
)]
#[case::bonus_example6(
    "Somewhere in Belfast under a shelf store \
     but above the shelf realm there's an \
     elf on a shelf on a shelf on a shelf \
     on a elf on a shelf on a shelf on a \
     shelf on a shelf on a elf on a elf on \
     a elf on a shelf on a \
    ",
    StatusCode::OK,
    types::ElfShelfCountSummary {
        loose_elves: 16u64,
        bare_shelves: 2u64,
        shelved_elves: 8u64,
    },
)]
#[test_log::test(tokio::test)]
async fn test_challenge_six(
    service: TestService,
    #[case] text: &str,
    #[case] expected_status: StatusCode,
    #[case] expected_summary: types::ElfShelfCountSummary,
) -> anyhow::Result<()> {
    let response = service
        .resolve(Request::post("/6").body(Body::from(text.as_bytes().to_vec()))?)
        .await?;

    assert_eq!(
        expected_status,
        response.status(),
        "status[expected: {}, actual: {}]",
        expected_status,
        response.status(),
    );

    let summary: types::ElfShelfCountSummary =
        serde_json::from_slice(response.into_body().data().await.unwrap()?.as_ref())?;

    assert_eq!(
        &expected_summary, &summary,
        r#"{{"actual": {:?}, "expected": {:?}, "text": "{text}"}}"#,
        summary, expected_summary
    );

    Ok(())
}

/// Test that `decode_cookie` and `analyze_recipe` satisfy the conditions of
/// [CCH 2023 Challenge 7](https://console.shuttle.rs/cch/challenge/7)
#[rstest]
#[case::challenge_example(
    "/7/decode",
    "eyJmbG91ciI6MTAwLCJjaG9jb2xhdGUgY2hpcHMiOjIwfQ==",
    StatusCode::OK,
    "{\"flour\":100,\"chocolate chips\":20}"
)]
#[case::bonus_example(
    "/7/bake",
    "eyJyZWNpcGUiOnsiZmxvdXIiOjk1LCJzdWdhciI6NTAsImJ1\
     dHRlciI6MzAsImJha2luZyBwb3dkZXIiOjEwLCJjaG9jb2xh\
     dGUgY2hpcHMiOjUwfSwicGFudHJ5Ijp7ImZsb3VyIjozODUs\
     InN1Z2FyIjo1MDcsImJ1dHRlciI6MjEyMiwiYmFraW5nIHBv\
     d2RlciI6ODY1LCJjaG9jb2xhdGUgY2hpcHMiOjQ1N319\
    ",
    StatusCode::OK,
    r#"{
      "cookies": 4,
      "pantry": {
        "flour": 5,
        "sugar": 307,
        "butter": 2002,
        "baking powder": 825,
        "chocolate chips": 257
      }
    }
    "#
)]
#[ignore = "not implemented yet"]
#[case::second_bonus_example(
    "/7/bake",
    "eyJyZWNpcGUiOnsic2xpbWUiOjl9LCJwYW50cnkiO\
    nsiY29iYmxlc3RvbmUiOjY0LCJzdGljayI6IDR9fQ==",
    StatusCode::OK,
    r#"{
      "cookies": 0,
      "pantry": {
        "cobblestone": 64,
        "stick": 4
      }
    }
    "#
)]
#[test_log::test(tokio::test)]
async fn test_challenge_seven(
    service: TestService,
    #[case] url: &str,
    #[case] cookie: &str,
    #[case] expected_status: StatusCode,
    #[case] expected_content: RecipeOrBakeResult,
) -> anyhow::Result<()> {
    let response = service
        .resolve(Request::get(url).header(headers::COOKIE, format!("recipe={cookie}")))
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
        RecipeOrBakeResult::Recipe(expected_recipe) => {
            let actual_recipe = serde_json::from_slice::<types::CookieRecipe>(content.as_ref())?;

            assert_eq!(
                expected_recipe, actual_recipe,
                "recipe[expected: {:?}, actual: {:?}]",
                expected_recipe, actual_recipe,
            );
        }
        RecipeOrBakeResult::BakeResult(expected_result) => {
            let actual_result =
                serde_json::from_slice::<types::CookieRecipeInventory>(content.as_ref())?;

            assert_eq!(
                expected_result, actual_result,
                "result[expected: {:?}, actual: {:?}]",
                expected_result, actual_result,
            );
        }
    }

    Ok(())
}

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

// </editor-fold desc="// Tests ...">
