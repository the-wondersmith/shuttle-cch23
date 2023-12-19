//! ### CCH 2023 Day 7 Solutions
//!

// Standard Library Imports
use core::{
    cmp::PartialOrd,
    convert::{AsMut, AsRef},
    fmt::{Debug, Display, Formatter, Result as FormatResult},
    mem::discriminant as enum_variant,
    ops::{Deref, DerefMut, Not, Sub, SubAssign},
};

// Third-Party Imports
use axum::{
    async_trait,
    extract::{FromRequestParts, Json},
    http::{header::COOKIE, request::Parts, StatusCode},
};
use b64::{engine::general_purpose as base64, Engine};
use itertools::Itertools;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{map::Map as JsonObject, Value};

// <editor-fold desc="// Types ...">

/// A recipe detailing the required
/// ingredients to make one cookie
pub type CookieRecipe = CookieData;

/// A per-ingredient inventory of
/// the contents of Santa's pantry
pub type PantryInventory = CookieData;

type RecipeAnalysisResponse = (StatusCode, Json<CookieRecipeInventory>);

// <editor-fold desc="// CookieData ...">

/// TODO(the-wondersmith): documentation
#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CookieData(pub JsonObject<String, Value>);

impl Deref for CookieData {
    type Target = JsonObject<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CookieData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[allow(clippy::from_over_into)]
impl Into<Value> for CookieData {
    fn into(self) -> Value {
        Value::Object(self.0)
    }
}

impl Display for CookieData {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        write!(formatter, "{}", Self::_stringify(&self.0.clone().into()))
    }
}

impl AsMut<Self> for CookieData {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl AsRef<Self> for CookieData {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl TryFrom<Value> for CookieData {
    type Error = Value;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Object(instance) = value {
            Ok(Self(instance))
        } else {
            Err(value)
        }
    }
}

impl<AsCookieData: AsRef<Self>> Sub<AsCookieData> for CookieData {
    type Output = Self;

    fn sub(self, other: AsCookieData) -> Self::Output {
        Self::Output::_sub(self, other)
    }
}

impl<'data, AsCookieData: AsRef<CookieData>> Sub<AsCookieData> for &'data CookieData {
    type Output = CookieData;

    fn sub(self: &'data CookieData, other: AsCookieData) -> Self::Output {
        Self::Output::_sub(self, other)
    }
}
impl<'data, AsCookieData: AsRef<CookieData>> SubAssign<AsCookieData> for &'data mut CookieData {
    fn sub_assign(&mut self, other: AsCookieData) {
        CookieData::_sub_assign(self, other);
    }
}

impl CookieData {
    /// Set all ingredient fields to 0
    pub(super) fn clear(&mut self) {
        self.retain(|_, _| false)
    }

    /// Check if a "pantry" is "empty"
    pub(super) fn is_empty(&self) -> bool {
        Self::_is_empty(&self.0)
    }

    fn _is_empty(object: &JsonObject<String, Value>) -> bool {
        object.keys().len() == 0
            || object
                .iter()
                .any(|(_, value)| match value {
                    Value::Null => false,
                    Value::Bool(flag) => *flag,
                    Value::Number(value) => value
                        .as_u64()
                        .map(|value| value.gt(&0u64))
                        .or_else(|| value.as_i64().map(|value| value.gt(&0i64)))
                        .or_else(|| value.as_f64().map(|value| value.gt(&0.0)))
                        .is_some_and(|value| value),
                    Value::String(value) => value.is_empty().not(),
                    Value::Array(value) => value.is_empty().not(),
                    Value::Object(value) => Self::_is_empty(value).not(),
                })
                .not()
    }

    fn _stringify(data: &Value) -> String {
        match data {
            Value::Null => String::from("null"),
            Value::Bool(value) => value.to_string(),
            Value::String(value) => value.to_string(),
            Value::Number(value) => value.to_string(),
            Value::Array(value) => {
                format!("[{}]", value.iter().map(Self::_stringify).join(", "))
            }
            Value::Object(mapping) => {
                format!(
                    "{{{}}}",
                    mapping
                        .iter()
                        .map(|(key, value)| format!(
                            "{}: {}",
                            key.replace(' ', "_"),
                            Self::_stringify(value)
                        ))
                        .join(", ")
                )
            }
        }
    }

    /// "Subtract" the right instance from the left instance
    fn _sub<Left: AsRef<Self>, Right: AsRef<Self>>(left: Left, right: Right) -> Self {
        let (left, right) = (left.as_ref(), right.as_ref());

        let mut instance = JsonObject::<String, Value>::new();

        for (key, l_value, r_value) in Self::_intersection(left, right) {
            if matches!((l_value, r_value), (Value::Number(_), Value::Number(_))) {
                if let (Some(l_value), Some(r_value)) = (l_value.as_u64(), r_value.as_u64()) {
                    instance[key] = Value::from(l_value.saturating_sub(r_value));
                } else if let (Some(l_value), Some(r_value)) = (l_value.as_i64(), r_value.as_i64())
                {
                    instance[key] = Value::from(l_value - r_value);
                } else if let (Some(l_value), Some(r_value)) = (l_value.as_f64(), r_value.as_f64())
                {
                    instance[key] = Value::from(l_value - r_value);
                }
            } else {
                tracing::warn!(
                    "Unsupported value type combination for \
                    in-place subtraction: {{l_value: {:?}, r_value: {:?}}}",
                    enum_variant(l_value),
                    enum_variant(r_value),
                );
            }
        }

        Self(instance)
    }

    /// Determine if the right hand instance can be "subtracted" from the left hand
    /// in full, that is - without potentially causing an "underflow" condition
    pub fn _can_sub<Left: AsRef<Self>, Right: AsRef<Self>>(left: Left, right: Right) -> bool {
        Self::_intersection(left.as_ref(), right.as_ref())
            .any(|(_, left, right)| {
                if let (Some(left_value), Some(right_value)) = (left.as_u64(), right.as_u64()) {
                    left_value < right_value
                } else if let (Some(left_value), Some(right_value)) =
                    (left.as_i64(), right.as_i64())
                {
                    left_value < right_value
                } else if let (Some(left_value), Some(right_value)) =
                    (left.as_f64(), right.as_f64())
                {
                    left_value < right_value
                } else {
                    true
                }
            })
            .not()
    }

    /// Get the key/value pairs that exist in both of the supplied
    /// JSON objects if and only if the value is of the same type
    /// on both "sides"
    fn _intersection<'left, 'right>(
        left: &'left Self,
        right: &'right Self,
    ) -> impl Iterator<Item = (&'left String, &'left Value, &'right Value)> {
        left.iter().filter_map(|(key, l_val)| {
            right.get(key).and_then(|r_val| {
                if enum_variant(l_val) == enum_variant(r_val) {
                    Some((key, l_val, r_val))
                } else {
                    None
                }
            })
        })
    }

    /// Perform an in-place subtraction of the right hand instance from the left
    fn _sub_assign<AsCookieData: AsRef<Self>>(&mut self, other: AsCookieData) {
        let other = other.as_ref();

        let mut computed: Vec<(String, Value)> = Vec::new();

        for (key, left, right) in Self::_intersection(self, other) {
            if let (Some(l_value), Some(r_value)) = (left.as_u64(), right.as_u64()) {
                computed.push((
                    key.to_string(),
                    Value::from(l_value.saturating_sub(r_value)),
                ));
            } else if let (Some(l_value), Some(r_value)) = (left.as_i64(), right.as_i64()) {
                computed.push((key.to_string(), Value::from(l_value - r_value)));
            } else if let (Some(l_value), Some(r_value)) = (left.as_f64(), right.as_f64()) {
                computed.push((key.to_string(), Value::from(l_value - r_value)));
            } else {
                tracing::warn!(
                    "Unsupported value type combination for \
                    in-place subtraction: {{left: {:?}, right: {:?}}}",
                    enum_variant(left),
                    enum_variant(right),
                );
            }
        }

        for (key, computed_value) in computed {
            self[&key] = computed_value;
        }
    }
}

// </editor-fold desc="// CookieData ...">

// <editor-fold desc="// CookieRecipeInventory ...">

/// A cookie recipe detailing the required
/// per-cookie amount of each ingredient,
/// along with a, inventory detailing how
/// much of each ingredient remains in
/// Santa's pantry post-baking
#[derive(derive_more::Display)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug, Default, Serialize, Deserialize)]
#[display(
    fmt = r#"{{cookies: {}, recipe: {}, pantry: {}}}"#,
    cookies,
    recipe,
    pantry
)]
pub struct CookieRecipeInventory {
    /// The absolute total number of cookies
    /// that can be baked according to the
    /// associated recipe with the ingredients
    /// in the associated pantry inventory
    #[serde(default)]
    // #[serde(skip_serializing_if = "is_zero")]
    pub cookies: u64,
    /// A recipe detailing the required
    /// ingredients to make one cookie
    #[serde(default)]
    #[serde(skip_serializing_if = "CookieRecipe::is_empty")]
    pub recipe: CookieData,
    /// A per-ingredient inventory
    /// of the contents of Santa's
    /// pantry post-baking
    #[serde(default)]
    pub pantry: CookieData,
}

impl CookieRecipeInventory {
    /// Calculate how many cookies can be baked according
    /// to a given recipe and inventory of ingredients.
    /// Additionally, update the pantry's inventory to
    /// reflect the ingredients consumed by the baking process.
    ///
    /// ### Example:
    ///
    /// Given the following recipe and pantry inventory:
    ///
    /// ```json
    /// {
    ///    "recipe": {
    ///      "flour": 95,
    ///      "sugar": 50,
    ///      "butter": 30,
    ///      "baking powder": 10,
    ///      "chocolate chips": 50
    ///    },
    ///    "pantry": {
    ///      "flour": 385,
    ///      "sugar": 507,
    ///      "butter": 2122,
    ///      "baking powder": 865,
    ///      "chocolate chips": 457
    ///    }
    /// }
    /// ```
    ///
    /// The resulting `CookieRecipeInventory` state would be:
    ///
    /// ```json
    /// {
    ///   "cookies": 4,
    ///   "pantry": {
    ///     "flour": 5,
    ///     "sugar": 307,
    ///     "butter": 2002,
    ///     "baking powder": 825,
    ///     "chocolate chips": 257
    ///   }
    /// }
    /// ```
    #[tracing::instrument(skip(self), fields(after, before))]
    pub fn bake(mut self) -> Self {
        // Record the pre-bake state as part of the current span.
        tracing::Span::current().record("before", format!("{}", &self).as_str());

        if self.recipe.is_empty() {
            tracing::warn!(r#"Declining to "re-bake" previously recipe/pantry"#);
            return self;
        }

        self.cookies = 0;

        loop {
            if PantryInventory::_can_sub(self.pantry.as_ref(), self.recipe.as_ref()) {
                PantryInventory::_sub_assign(self.pantry.as_mut(), self.recipe.as_ref());
                self.cookies += 1;
            } else {
                break;
            }
        }

        self.recipe.clear();

        // Record the post-bake state as part of the current span.
        tracing::Span::current().record("after", format!("{}", &self).as_str());

        self
    }
}

// </editor-fold desc="// CookieRecipeInventory ...">

// <editor-fold desc="// CookieRecipeHeader ...">

/// [`axum` extractor](axum::extract) for
/// variadic path values (e.g. `/endpoint/*values`)
#[derive(Debug)]
pub struct CookieRecipeHeader<Recipe>(pub Recipe);

#[async_trait]
impl<State, Recipe> FromRequestParts<State> for CookieRecipeHeader<Recipe>
where
    State: Send + Sync,
    Recipe: Debug + DeserializeOwned,
{
    type Rejection = (StatusCode, String);

    #[tracing::instrument(skip_all, fields(cookie))]
    async fn from_request_parts(
        parts: &mut Parts,
        _: &State,
    ) -> anyhow::Result<Self, Self::Rejection> {
        parts
            .headers
            .get(COOKIE)
            .ok_or((
                StatusCode::BAD_REQUEST,
                String::from(r#""cookie" header missing"#),
            ))
            .and_then(|header| {
                tracing::Span::current().record("cookie", format!("{header:?}").as_str());
                let header = header.as_bytes();

                match (
                    header.starts_with(b"recipe="),
                    header.strip_prefix(b"recipe="),
                ) {
                    (true, Some(value)) => Ok(value),
                    _ => {
                        tracing::warn!(r#"cookie header present but missing "recipe=" prefix"#);
                        Err((
                            StatusCode::EXPECTATION_FAILED,
                            format!(
                                r#"missing "recipe=" prefix: {}"#,
                                String::from_utf8_lossy(header)
                            ),
                        ))
                    }
                }
            })
            .and_then(|encoded| {
                base64::STANDARD.decode(encoded).map_err(|error| {
                    let error = error.to_string();

                    tracing::warn!("un-decodable cookie header: {}", &error);
                    (StatusCode::EXPECTATION_FAILED, error)
                })
            })
            .and_then(|decoded| {
                serde_json::from_slice::<Recipe>(decoded.as_slice()).map_err(|error| {
                    let error = error.to_string();

                    tracing::warn!("un-decodable cookie header: {}", &error);

                    (StatusCode::UNPROCESSABLE_ENTITY, error)
                })
            })
            .map(Self)
    }
}

// </editor-fold desc="// CookieRecipeHeader ...">

// </editor-fold desc="// Types ...">

/// Complete [Day 7: Challenge](https://console.shuttle.rs/cch/challenge/7#:~:text=⭐)
#[tracing::instrument(skip_all)]
pub async fn decode_cookie_recipe(
    CookieRecipeHeader(recipe): CookieRecipeHeader<Value>,
) -> Json<Value> {
    Json(recipe)
}

/// Complete [Day 7: Bonus](https://console.shuttle.rs/cch/challenge/7#:~:text=🎁)
#[tracing::instrument(skip_all, fields(request, response))]
pub async fn bake_cookies_from_recipe_and_pantry(
    CookieRecipeHeader(data): CookieRecipeHeader<CookieRecipeInventory>,
) -> RecipeAnalysisResponse {
    tracing::Span::current().record("request", format!("{}", &data).as_str());

    let data = data.bake();

    tracing::Span::current().record("response", format!("{}", &data).as_str());

    tracing::info!("\u{1F6CE}\u{FE0F}\u{3000}\u{1F36A}");

    (StatusCode::OK, Json(data))
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
    use super::{CookieRecipe, CookieRecipeInventory};
    use crate::utils::{service, TestService};

    #[derive(Debug)]
    enum RecipeOrBakeResult {
        /// Decoded cookie recipe returned
        /// by the `/7/decode` endpoint
        Recipe(CookieRecipe),
        /// "Baking" summary returned
        /// by the `/7/bake` endpoint
        BakeResult(CookieRecipeInventory),
    }

    impl FromStr for RecipeOrBakeResult {
        type Err = SerdeJsonError;

        fn from_str(string: &str) -> Result<Self, Self::Err> {
            serde_json::from_str::<CookieRecipe>(string)
                .map(Self::Recipe)
                .map_or_else(
                    |_| serde_json::from_str::<CookieRecipeInventory>(string).map(Self::BakeResult),
                    Ok,
                )
        }
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
                let actual_recipe = serde_json::from_slice::<CookieRecipe>(content.as_ref())?;

                assert_eq!(
                    expected_recipe, actual_recipe,
                    "recipe[expected: {:?}, actual: {:?}]",
                    expected_recipe, actual_recipe,
                );
            }
            RecipeOrBakeResult::BakeResult(expected_result) => {
                let actual_result =
                    serde_json::from_slice::<CookieRecipeInventory>(content.as_ref())?;

                assert_eq!(
                    expected_result, actual_result,
                    "result[expected: {:?}, actual: {:?}]",
                    expected_result, actual_result,
                );
            }
        }

        Ok(())
    }
}
