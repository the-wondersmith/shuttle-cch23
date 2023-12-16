//! ### CCH 2023 Day 7 Solutions
//!

// Third-Party Imports
use axum::{extract::Json, http::StatusCode};
use serde_json::Value;

// Crate-Level Imports
use crate::types::{CookieRecipeHeader, CookieRecipeInventory, RecipeAnalysisResponse};

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
