//! ## Custom Types
//!

// Standard Library Imports
use core::{
    cmp::PartialOrd,
    convert::{AsMut, AsRef},
    fmt::{Debug, Display, Formatter, Result as FormatResult},
    mem::discriminant as enum_variant,
    ops::{Deref, DerefMut, Not, Sub, SubAssign},
};
use std::{
    boxed::Box,
    collections::{BTreeMap, HashMap},
    env::{set_var as set_env_var, temp_dir, var as get_env_var},
    path::PathBuf as FilePathBuf,
};

// Third-Party Imports
use axum::http::StatusCode;
use axum::{
    async_trait,
    extract::{rejection::PathRejection, FromRef, FromRequestParts, Path},
    http::{header::COOKIE, request::Parts},
    Json,
};
#[allow(unused_imports)]
use axum_template::{
    engine::{Engine as HandlebarsEngine, HandlebarsError},
    Key, RenderHtml,
};
use b64::{engine::general_purpose as base64, Engine};
use handlebars::{Handlebars, TemplateError};
use itertools::Itertools;
use num_traits::cast::FromPrimitive;
use serde::{de::DeserializeOwned, ser::Error, Deserialize, Serialize};
use serde_json::{map::Map as JsonObject, value::Value};
use shuttle_persist::{PersistError as PersistenceError, PersistInstance as Persistence};
use shuttle_secrets::SecretStore;
use sqlx::{error::Error as DbError, postgres::PgQueryResult};

pub(super) type TemplateEngine = HandlebarsEngine<Handlebars<'static>>;
pub(super) type RecipeAnalysisResponse = (StatusCode, Json<CookieRecipeInventory>);
pub(super) type NonNumericPacketIdResponse = (StatusCode, Json<HashMap<String, Vec<Value>>>);

/// Determine if the supplied value
/// is actually (or effectively) zero
#[inline]
fn is_zero<T: Display>(value: T) -> bool {
    value.to_string() == "0"
}

// <editor-fold desc="// ShuttleAppState ...">

/// The service's "shared" state
#[derive(Clone, Debug, FromRef)]
pub struct ShuttleAppState {
    /// A pool of connections to the
    /// service's PostgreSQL database
    pub db: sqlx::PgPool,
    /// A pre-configured Handlebars
    /// templating engine instance
    pub templates: TemplateEngine,
    /// The service's instance-independent
    /// persistent key-value store
    pub persistence: Persistence,
}

//noinspection RsReplaceMatchExpr
impl ShuttleAppState {
    /// Initialize the service's state
    #[tracing::instrument(skip_all)]
    pub fn initialize(
        db: sqlx::PgPool,
        secrets: Option<SecretStore>,
        templates: Option<TemplateEngine>,
        persistence: Option<Persistence>,
    ) -> anyhow::Result<Self> {
        Self::_initialize_secrets(secrets);

        let templates = templates.map_or_else(
            Self::_default_template_engine,
            Result::<TemplateEngine, Box<TemplateError>>::Ok,
        )?;

        let persistence = persistence.map_or_else(
            Self::_default_persistence,
            Result::<Persistence, PersistenceError>::Ok,
        )?;

        Ok(Self {
            db,
            templates,
            persistence,
        })
    }

    #[cfg_attr(tarpaulin, coverage(off))]
    #[cfg_attr(tarpaulin, tarpaulin::skip)]
    fn _default_secrets() -> SecretStore {
        SecretStore::new(BTreeMap::new())
    }

    #[cfg_attr(tarpaulin, coverage(off))]
    #[cfg_attr(tarpaulin, tarpaulin::skip)]
    fn _default_template_engine() -> Result<TemplateEngine, Box<TemplateError>> {
        let mut engine = Handlebars::new();

        if get_env_var("SHUTTLE").is_ok_and(|value| &value == "true") {
            engine.set_dev_mode(true);
        }

        engine
            .register_templates_directory(
                ".tpl",
                FilePathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")),
            )
            .map_err(Box::from)
            .map(|_| TemplateEngine::from(engine))
    }

    fn _initialize_secrets(secrets: Option<SecretStore>) -> SecretStore {
        let secrets = secrets.unwrap_or_else(Self::_default_secrets);

        if let Some(path) = get_env_var("CCH23_PERSISTENCE_DIR")
            .ok()
            .or_else(|| secrets.get("PERSISTENCE_DIR"))
        {
            set_env_var("CCH23_PERSISTENCE_DIR", path);
        }

        secrets
    }

    #[cfg_attr(tarpaulin, coverage(off))]
    #[cfg_attr(tarpaulin, tarpaulin::skip)]
    fn _default_persistence() -> anyhow::Result<Persistence, PersistenceError> {
        let path = get_env_var("CCH23_PERSISTENCE_DIR")
            .ok()
            .map_or_else(
                || temp_dir().join("shuttle-cch23").join("persistence"),
                FilePathBuf::from,
            )
            .canonicalize()
            .map_err(PersistenceError::CreateFolder)?;

        Persistence::new(path)
    }
}

// </editor-fold desc="// ShuttleAppState ...">

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

/// A recipe detailing the required
/// ingredients to make one cookie
pub type CookieRecipe = CookieData;

/// A per-ingredient inventory of
/// the contents of Santa's pantry
pub type PantryInventory = CookieData;

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

// <editor-fold desc="// ElfShelfCountSummary ...">

/// Custom struct for responding to elf/shelf count
/// requests for [Day 6](https://console.shuttle.rs/cch/challenge/6)
#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ElfShelfCountSummary {
    /// The count of how many times the literal
    /// string "elf" appears in the source text
    #[serde(alias = "elf")]
    #[serde(rename(serialize = "elf"))]
    pub loose_elves: u64,
    /// The count of how many times the literal string
    /// "elf on a shelf" appears in the source text
    #[serde(default)]
    #[serde(alias = "elf on a shelf")]
    #[serde(rename(serialize = "elf on a shelf"))]
    pub shelved_elves: u64,
    /// The number of shelves that don't have an elf on them -
    /// that is, the number of strings "shelf" that are not
    /// preceded by the string "elf on a ".
    #[serde(default)]
    #[serde(alias = "shelf with no elf on it")]
    #[serde(rename(serialize = "shelf with no elf on it"))]
    pub bare_shelves: u64,
}

impl<T: AsRef<str>> From<T> for ElfShelfCountSummary {
    fn from(text: T) -> Self {
        let text = text.as_ref();

        // - The count of how many times the literal
        //   string "elf" appears in the source text
        // - The count of how many times the literal string
        //   "elf on a shelf" appears in the source text
        // - The number of shelves that don't have an elf on them -
        //   that is, the number of strings "shelf" that are not
        //   preceded by the string "elf on a ".

        let mut summary = Self::default();

        for idx in 0..text.len() {
            match &text[idx..] {
                segment if segment.starts_with("elf on a shelf") => {
                    // that's one loose elf
                    summary.loose_elves += 1;
                    // and one shelved elf
                    summary.shelved_elves += 1;
                }
                segment if segment.starts_with("elf") => {
                    summary.loose_elves += 1;
                }
                segment if segment.starts_with("shelf") => {
                    summary.bare_shelves += 1;
                }
                _ => (),
            }
        }

        // Adjust the count of shelves to exclude shelves with an elf
        summary.bare_shelves = u64::saturating_sub(summary.bare_shelves, summary.shelved_elves);

        summary
    }
}

// </editor-fold desc="// ElfShelfCountSummary ...">

// <editor-fold desc="// GiftOrder ...">

/// A gift order
#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GiftOrder {
    /// the order's sequential id
    pub id: u64,
    /// how many `{gift_name}`s were ordered
    pub quantity: u64,
    /// the gift's elf-readable name
    pub gift_name: String,
    /// the region to which the
    /// gift must be delivered
    pub region_id: u64,
}

impl GiftOrder {
    /// ...
    pub async fn insert(&self, db: &sqlx::PgPool) -> Result<PgQueryResult, DbError> {
        Self::insert_many([self].into_iter(), db).await
    }

    /// ...
    pub async fn insert_many<'orders, Orders: Iterator<Item = &'orders Self>>(
        orders: Orders,
        db: &sqlx::PgPool,
    ) -> Result<PgQueryResult, DbError> {
        sqlx::QueryBuilder::<sqlx::Postgres>::new(
            "INSERT INTO ORDERS (id, quantity, gift_name, region_id) ",
        )
        .push_values(orders, |mut builder, order| {
            builder
                .push_bind(order.id as i64)
                .push_bind(order.quantity as i64)
                .push_bind(order.gift_name.clone())
                .push_bind(order.region_id as i64);
        })
        .build()
        .execute(db)
        .await
    }

    /// ...
    pub async fn total_ordered(db: &sqlx::PgPool) -> Result<u64, DbError> {
        sqlx::query_scalar::<_, i64>("SELECT SUM(quantity) FROM orders")
            .fetch_one(db)
            .await
            .and_then(|count| {
                u64::from_i64(count).ok_or(DbError::Decode(
                    anyhow::anyhow!("bad count value: {count}").into(),
                ))
            })
    }

    /// ...
    pub async fn most_popular(db: &sqlx::PgPool) -> Result<Option<(String, i64)>, DbError> {
        sqlx::query_as(
            r#"
            SELECT
                gift_name,
                SUM(quantity) as popularity
            FROM
                orders
            GROUP BY
                gift_name
            ORDER BY
                popularity
            DESC
            LIMIT 1
        "#,
        )
        .fetch_optional(db)
        .await
    }
}

// </editor-fold desc="// GiftOrder ...">
