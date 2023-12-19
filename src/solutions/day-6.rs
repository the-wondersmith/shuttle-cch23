//! ### CCH 2023 Day 6 Solutions
//!

// Standard Library Imports
use core::{convert::AsRef, fmt::Debug};

// Third-Party Imports
use axum::Json;
#[allow(unused_imports)]
use axum_template::{
    engine::{Engine as HandlebarsEngine, HandlebarsError},
    Key, RenderHtml,
};
use serde::{Deserialize, Serialize};

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

/// Complete [Day 6: Task + Bonus](https://console.shuttle.rs/cch/challenge/6#:~:text=🎄)
#[tracing::instrument(ret)]
pub async fn count_elves(text: String) -> Json<ElfShelfCountSummary> {
    Json(ElfShelfCountSummary::from(text))
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
    use super::ElfShelfCountSummary;
    use crate::utils::{service, TestService};

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
        ElfShelfCountSummary {
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
        ElfShelfCountSummary {
            loose_elves: 5u64,
            bare_shelves: 1u64,
            shelved_elves: 1u64,
        },
    )]
    #[case::bonus_example2(
        "elf elf elf",
        StatusCode::OK,
        ElfShelfCountSummary {
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
        ElfShelfCountSummary {
            loose_elves: 6u64,
            bare_shelves: 0u64,
            shelved_elves: 0u64,
        },
    )]
    #[case::bonus_example4(
        "elf elf elf on a shelf",
        StatusCode::OK,
        ElfShelfCountSummary {
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
        ElfShelfCountSummary {
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
        ElfShelfCountSummary {
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
        #[case] expected_summary: ElfShelfCountSummary,
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

        let summary: ElfShelfCountSummary =
            serde_json::from_slice(response.into_body().data().await.unwrap()?.as_ref())?;

        assert_eq!(
            &expected_summary, &summary,
            r#"{{"actual": {:?}, "expected": {:?}, "text": "{text}"}}"#,
            summary, expected_summary
        );

        Ok(())
    }
}
