//! ## Custom Types
//!

// Standard Library Imports
use core::{
    cmp::PartialOrd,
    convert::{AsMut, AsRef},
    fmt::{Debug, Display, Formatter, Result as FormatResult},
    ops::{Not, Sub, SubAssign},
};
use std::collections::HashMap;

// Third-Party Imports
use axum::http::StatusCode;
use axum::{
    async_trait,
    extract::{rejection::PathRejection, FromRequestParts, Path},
    http::{header::COOKIE, request::Parts},
    Json,
};
use b64::{engine::general_purpose as base64, Engine};
use num::FromPrimitive;
use serde::{de::DeserializeOwned, ser::Error, Deserialize, Serialize};
use serde_json::value::Value;
use unicode_normalization::UnicodeNormalization;

/// Unicode "missing character"
const UFFFD: &str = "\u{FFFD}";

/// TODO(the-wondersmith): documentation
pub(super) type NonNumericPacketIdResponse = (StatusCode, Json<HashMap<String, Vec<Value>>>);
/// TODO(the-wondersmith): documentation
pub(super) type EmptyRecipeOrPantryResponse = (StatusCode, Json<CookieRecipeInventory>);

/// Determine if the supplied value
/// is actually (or effectively) zero
#[inline]
fn is_zero<T: Display>(value: T) -> bool {
    value.to_string() == "0"
}

// <editor-fold desc="// CookieRecipeInventory ...">

#[allow(missing_docs)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct _CookieData {
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub flour: u64,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub sugar: u64,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub butter: u64,
    #[serde(default)]
    #[serde(rename = "baking powder")]
    #[serde(skip_serializing_if = "is_zero")]
    pub baking_powder: u64,
    #[serde(default)]
    #[serde(rename = "chocolate chips")]
    #[serde(skip_serializing_if = "is_zero")]
    pub chocolate_chips: u64,
}

/// A recipe detailing the required
/// ingredients to make one cookie
pub type CookieRecipe = _CookieData;

/// A per-ingredient inventory of
/// the contents of Santa's pantry
pub type PantryInventory = _CookieData;

/// A cookie recipe detailing the required
/// per-cookie amount of each ingredient,
/// along with a, inventory detailing how
/// much of each ingredient remains in
/// Santa's pantry post-baking
#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CookieRecipeInventory {
    /// The absolute total number of cookies
    /// that can be baked according to the
    /// associated recipe with the ingredients
    /// in the associated pantry inventory
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub cookies: u64,
    /// A recipe detailing the required
    /// ingredients to make one cookie
    #[serde(default)]
    #[serde(skip_serializing_if = "CookieRecipe::is_empty")]
    pub recipe: CookieRecipe,
    /// A per-ingredient inventory
    /// of the contents of Santa's
    /// pantry post-baking
    #[serde(default)]
    pub pantry: PantryInventory,
}

impl _CookieData {
    /// Set all ingredient fields to 0
    pub(super) fn clear(&mut self) {
        [
            self.flour,
            self.sugar,
            self.butter,
            self.baking_powder,
            self.chocolate_chips,
        ] = [0u64; 5];
    }

    /// Check if a "pantry" is "empty"
    pub(super) fn is_empty(&self) -> bool {
        [
            self.flour,
            self.sugar,
            self.butter,
            self.baking_powder,
            self.chocolate_chips,
        ]
        .iter()
        .any(|item| u64::gt(item, &0u64))
        .not()
    }

    /// "Subtract" the right instance from the left instance
    fn _sub<Left: AsRef<Self>, Right: AsRef<Self>>(left: Left, right: Right) -> Self {
        let (left, right) = (left.as_ref(), right.as_ref());

        Self {
            flour: left.flour.saturating_sub(right.flour),
            sugar: left.sugar.saturating_sub(right.sugar),
            butter: left.butter.saturating_sub(right.butter),
            baking_powder: left.baking_powder.saturating_sub(right.baking_powder),
            chocolate_chips: left.chocolate_chips.saturating_sub(right.chocolate_chips),
        }
    }

    /// Determine if the right hand instance can be "subtracted" from the left hand
    /// in full, that is - without potentially causing an "underflow" condition
    fn _can_sub<Left: AsRef<Self>, Right: AsRef<Self>>(left: Left, right: Right) -> bool {
        let (left, right) = (left.as_ref(), right.as_ref());

        [
            (left.flour, right.flour),
            (left.sugar, right.sugar),
            (left.butter, right.butter),
            (left.baking_powder, right.baking_powder),
            (left.chocolate_chips, right.chocolate_chips),
        ]
        .iter()
        .any(|pair| u64::le(&pair.0, &pair.1))
        .not()
    }

    /// Perform an in-place subtraction of the right hand instance from the left
    fn _sub_assign<AsCookieData: AsRef<Self>>(&mut self, other: AsCookieData) {
        let other = other.as_ref();

        self.flour = self.flour.saturating_sub(other.flour);
        self.sugar = self.sugar.saturating_sub(other.sugar);
        self.butter = self.butter.saturating_sub(other.butter);
        self.baking_powder = self.baking_powder.saturating_sub(other.baking_powder);
        self.chocolate_chips = self.chocolate_chips.saturating_sub(other.chocolate_chips);
    }
}

impl AsMut<Self> for _CookieData {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl AsRef<Self> for _CookieData {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<AsCookieData: AsRef<Self>> Sub<AsCookieData> for _CookieData {
    type Output = Self;

    fn sub(self, other: AsCookieData) -> Self::Output {
        Self::Output::_sub(self, other)
    }
}

impl<'data, AsCookieData: AsRef<_CookieData>> Sub<AsCookieData> for &'data _CookieData {
    type Output = _CookieData;

    fn sub(self: &'data _CookieData, other: AsCookieData) -> Self::Output {
        Self::Output::_sub(self, other)
    }
}
impl<'data, AsCookieData: AsRef<_CookieData>> SubAssign<AsCookieData> for &'data mut _CookieData {
    fn sub_assign(&mut self, other: AsCookieData) {
        _CookieData::_sub_assign(self, other);
    }
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
    #[tracing::instrument(
        skip(self),
        fields(
            after = Option::<String>::None,
            before = Option::<String>::None,
        ),
    )]
    pub fn bake(mut self) -> Self {
        // Record the pre-bake state as part of the current span.
        tracing::Span::current().record("before", serde_json::to_string(&self).ok());

        if self.recipe.is_empty() {
            tracing::warn!(r#"Declining to "re-bake" previously recipe/pantry"#);
            return self;
        }

        self.cookies = 0;

        while PantryInventory::_can_sub(self.pantry.as_ref(), self.recipe.as_ref()) {
            self.cookies += 1;
            PantryInventory::_sub_assign(self.pantry.as_mut(), self.recipe.as_ref());
        }

        self.recipe.clear();

        // Record the post-bake state as part of the current span.
        tracing::Span::current().record("after", serde_json::to_string(&self).ok());

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
    type Rejection = (StatusCode, Json<Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        _: &State,
    ) -> anyhow::Result<Self, Self::Rejection> {
        parts
            .headers
            .get(COOKIE)
            .ok_or((
                StatusCode::BAD_REQUEST,
                Json(Value::from(r#""cookie" header missing"#)),
            ))
            .and_then(|header| {
                let header = header.as_bytes();

                match (
                    header.starts_with(b"recipe="),
                    header.strip_prefix(b"recipe="),
                ) {
                    (true, Some(value)) => Ok(value),
                    _ => Err((
                        StatusCode::EXPECTATION_FAILED,
                        Json(Value::from(format!(
                            r#"missing "recipe=" prefix: {}"#,
                            String::from_utf8_lossy(header)
                        ))),
                    )),
                }
            })
            .and_then(|encoded| {
                base64::STANDARD.decode(encoded).map_err(|error| {
                    (
                        StatusCode::EXPECTATION_FAILED,
                        Json(Value::from(error.to_string())),
                    )
                })
            })
            .and_then(|decoded| {
                serde_json::from_slice::<Recipe>(decoded.as_slice()).map_err(|error| {
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(Value::from(error.to_string())),
                    )
                })
            })
            .map(Self)
    }
}

// </editor-fold desc="// CookieRecipeHeader ...">

// <editor-fold desc="// VariadicPathValues ...">

/// [`axum` extractor](axum::extract) for
/// variadic path values (e.g. `/endpoint/*values`)
#[derive(Debug)]
pub struct VariadicPathValues(pub Vec<Value>);

#[async_trait]
impl<State: Send + Sync> FromRequestParts<State> for VariadicPathValues {
    type Rejection = PathRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &State,
    ) -> anyhow::Result<Self, Self::Rejection> {
        let values = <Path<String> as FromRequestParts<State>>::from_request_parts(parts, state)
            .await?
            .split('/')
            .map(|part| {
                serde_json::from_str::<Value>(part)
                    .map_or_else(|_| Value::from(part), |value| value)
            })
            .collect::<Vec<Value>>();

        Ok(Self(values))
    }
}

// </editor-fold desc="// VariadicPathValues ...">

// <editor-fold desc="// ReindeerStats ...">

/// Custom struct for extracting data from the body
/// of requests to the endpoint for [Challenge 4: Task](https://console.shuttle.rs/cch/challenge/4#:~:text=⭐)
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
        let mut widest: i64 = 0;
        let mut fastest: f64 = 0.0;
        let mut tallest: i64 = 0;
        let mut consumed: i64 = 0;
        let mut strongest: i64 = 0;
        let mut magic_power: i64 = 0;

        let mut summary: HashMap<String, String> = HashMap::new();

        for reindeer in stats {
            if reindeer.speed > fastest {
                fastest = reindeer.speed;
                summary
                    .entry("fastest".into())
                    .or_insert_with(|| reindeer.name.clone());
            }

            if reindeer.height > tallest {
                tallest = reindeer.height;
                summary
                    .entry("tallest".into())
                    .or_insert_with(|| reindeer.name.clone());
            }

            if reindeer.strength > strongest {
                strongest = reindeer.strength;
                summary
                    .entry("strongest".into())
                    .or_insert_with(|| reindeer.name.clone());
            }

            if reindeer.antler_width > widest {
                widest = reindeer.antler_width;
                summary
                    .entry("widest".into())
                    .or_insert_with(|| reindeer.name.clone());
            }

            if reindeer.snow_magic_power > magic_power {
                magic_power = reindeer.snow_magic_power;
                summary
                    .entry("magician".into())
                    .or_insert_with(|| reindeer.name.clone());
            }

            if reindeer.candies_eaten_yesterday > consumed {
                consumed = reindeer.candies_eaten_yesterday;
                summary
                    .entry("consumer".into())
                    .or_insert_with(|| reindeer.name.clone());
            }
        }

        for (key, value) in &mut summary {
            *value = match key.as_str() {
                "tallest" => format!("{value} is standing tall at {tallest} cm"),
                "widest" => format!("{value} is the thiccest boi at {widest} cm"),
                "magician" => format!("{value} could blast you away with a snow magic power of {fastest}"),
                "fastest" => format!("{value} absolutely guzzles Rust-Eze\u{2122} to maintain his speed rating of {fastest}"),
                "consumer" => format!("{value} is an absolute slut for candy and consumed {consumed} pieces of it yesterday"),
                "strongest" => format!("{value} is the strongest reindeer around with an impressive strength rating of {strongest}"),
                _ => value.clone(),
            };
        }

        summary
    }
}

// </editor-fold desc="// ReindeerStats ...">

// <editor-fold desc="// ElfShelfCountSummary ...">

/// Custom struct for responding to elf/shelf count
/// requests for [Challenge 6](https://console.shuttle.rs/cch/challenge/6)
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ElfShelfCountSummary {
    /// The count of how many times the literal
    /// string "elf" appears in the source text
    #[serde(alias = "elf")]
    #[serde(rename(serialize = "elf"))]
    pub elves: u64,
    /// The count of how many times the literal string
    /// "elf on a shelf" appears in the source text
    #[serde(default)]
    #[serde(alias = "elf on a shelf")]
    #[serde(skip_serializing_if = "is_zero")]
    #[serde(rename(serialize = "elf on a shelf"))]
    pub shelved_elves: u64,
    /// The number of shelves that don't have an elf on them -
    /// that is, the number of strings "shelf" that are not
    /// preceded by the string "elf on a ".
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    #[serde(alias = "shelf with no elf on it")]
    #[serde(rename(serialize = "shelf with no elf on it"))]
    pub shelves: u64,
}

impl<T: AsRef<str>> From<T> for ElfShelfCountSummary {
    fn from(value: T) -> Self {
        let value = value.as_ref().nfkd().to_string();
        let shelved = value.replace("elf on a shelf", "\u{0}");

        let elves =
            u64::from_usize(value.replace("elf", UFFFD).matches(UFFFD).count()).unwrap_or(u64::MAX);
        let bare_shelves = u64::from_usize(shelved.replace("shelf", UFFFD).matches(UFFFD).count())
            .unwrap_or(u64::MAX);
        let shelved_elves = u64::from_usize(shelved.matches('\u{0}').count()).unwrap_or(u64::MAX);

        Self {
            elves,
            shelves: bare_shelves,
            shelved_elves,
        }
    }
}

// </editor-fold desc="// ElfShelfCountSummary ...">
