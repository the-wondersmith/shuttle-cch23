#![allow(non_upper_case_globals)]
//! ### CCH 2023 Day 15 Solutions
//!

// Standard Library Imports
use std::iter::Iterator;
use std::{
    collections::HashMap,
    ops::{BitAnd, BitOr, Not},
    string::ToString,
};

// Third-Party Imports
use axum::{extract::Json, http::StatusCode};
use itertools::Itertools;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

// <editor-fold desc="// Type Aliases ...">

type EvaluationResponse = (StatusCode, Json<HashMap<String, String>>);
type ComplexEvaluationResult<'input> = Result<&'input str, (StatusCode, &'static str)>;
type NaughtyNiceEvaluationResponse = Result<EvaluationResponse, EvaluationResponse>;

// </editor-fold desc="// Type Aliases ...">

// <editor-fold desc="// Static Values ...">

// (0x2980..=0x2BFF)
static NICE_RANGE: Lazy<Vec<char>> = Lazy::new(|| {
    (0x2980..=0x2BFF)
        .filter_map(char::from_u32)
        .collect::<Vec<char>>()
});
static EMOJIS: Lazy<Vec<char>> = Lazy::new(|| {
    itertools::chain![
        0x0080..=0x02AF,
        0x0300..=0x03FF,
        0x0600..=0x06FF,
        0x0C00..=0x0C7F,
        0x1DC0..=0x1DFF,
        0x1E00..=0x1EFF,
        0x2000..=0x209F,
        0x20D0..=0x214F,
        0x2190..=0x23FF,
        0x2460..=0x25FF,
        0x2600..=0x27EF,
        0x2900..=0x29FF,
        0x2B00..=0x2BFF,
        0x2C60..=0x2C7F,
        0x2E00..=0x2E7F,
        0x3000..=0x303F,
        0xA490..=0xA4CF,
        0xE000..=0xF8FF,
        0xFE00..=0xFE0F,
        0xFE30..=0xFE4F,
        0x1F000..=0x1F02F,
        0x1F0A0..=0x1F0FF,
        0x1F100..=0x1F64F,
        0x1F680..=0x1F6FF,
        0x1F910..=0x1F96B,
        0x1F980..=0x1F9E0,
    ]
    .filter_map(char::from_u32)
    .filter(|c| NICE_RANGE.contains(c).not())
    .collect::<Vec<char>>()
});

const NICE: fn() -> Result<EvaluationResponse, EvaluationResponse> = || {
    Ok((
        StatusCode::OK,
        Json(HashMap::from([("result".to_string(), "nice".to_string())])),
    ))
};

const NAUGHTY: fn() -> Result<EvaluationResponse, EvaluationResponse> = || {
    Err((
        StatusCode::BAD_REQUEST,
        Json(HashMap::from([(
            "result".to_string(),
            "naughty".to_string(),
        )])),
    ))
};

const vowels: [char; 6] = ['a', 'e', 'i', 'o', 'u', 'y'];
const blacklist: [(char, char); 4] = [('a', 'b'), ('c', 'd'), ('p', 'q'), ('x', 'y')];

const valid_char: fn(char) -> bool = |c: char| c.is_ascii_alphabetic().bitor(c == ' ');
const valid_pair: fn((char, char)) -> bool =
    |pair: (char, char)| valid_char(pair.0).bitand(valid_char(pair.1));

// </editor-fold desc="// Static Values ...">

// <editor-fold desc="// NaughtyNiceEvaluation ...">

/// A naughty-or-nice string evaluation request
#[derive(derive_more::Display)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug, Default, Serialize, Deserialize)]
#[display(fmt = r#"{{input: {}}}"#, input)]
pub struct NaughtyNiceEvaluation {
    /// The string being evaluated
    pub input: String,
}

#[allow(clippy::needless_lifetimes)]
#[allow(dead_code, unused_variables)]
impl NaughtyNiceEvaluation {
    /// Perform a "simple" evaluation of the target string
    ///
    /// For a simple evaluation:
    ///   - Nice Strings:
    ///     - Must contain at least three vowels (aeiouy),
    ///     - at least one letter that appears twice in a row,
    ///     - must not contain the substrings: ab, cd, pq, or xy.
    ///
    ///  - Naughty Strings:
    ///    - Do not meet the criteria for nice strings.
    fn evaluate_simple(&self) -> NaughtyNiceEvaluationResponse {
        let password = &self.input;

        let (mut vowel_count, mut has_repeat) = (0u64, false);

        for pair in password.nfkd().tuple_windows::<(char, char)>() {
            if blacklist.contains(&pair) {
                tracing::Span::current()
                    .record("error", format!("blacklisted character pair: {pair:?}"));

                return NAUGHTY();
            }

            vowel_count += vowels.contains(&pair.1) as u64;
            has_repeat = has_repeat.bitor(valid_pair(pair).bitand(pair.0 == pair.1));
        }

        tracing::Span::current().record("vowels", vowel_count);

        let meets_vowel_count = 2 < vowel_count;

        if has_repeat && meets_vowel_count {
            return NICE();
        }

        let mut error = String::new();

        if !has_repeat {
            error.push_str("missing repeating characters");
        }

        if !meets_vowel_count {
            if !error.is_empty() {
                error.push_str(", and ");
            }

            error.push_str("below required vowel count");
        };

        tracing::Span::current().record("error", error);

        NAUGHTY()
    }

    /// Perform a "complex" evaluation of the target string
    ///
    /// For complex evaluations:
    ///   - Nice Strings (must adhere to all rules):
    ///     - 1: must be at least 8 characters long
    ///     - 2: must contain uppercase letters, lowercase letters, and digits
    ///     - 3: must contain at least 5 digits
    ///     - 4: all integers (sequences of consecutive digits) in the string must add up to 2023
    ///     - 5: must contain the letters j, o, and y in that order and in no other order
    ///     - 6: must contain a letter that repeats with exactly one other letter between them (like xyx)
    ///     - 7: must contain at least one unicode character in the range [U+2980, U+2BFF]
    ///     - 8: must contain at least one emoji
    ///     - 9: the hexadecimal representation of the sha256 hash of the string must end with an a
    ///
    ///   - Naughty Strings:
    ///     - do not meet the criteria for nice strings
    ///
    /// The target string will be checked against each rule above,
    /// in the listed order, returning the corresponding naughty/nice
    /// result, status code, and reason based on which rule was violated:
    ///
    /// | Rule broken | Status Code | Reason                 |
    /// | :---------- | :---------: | :--------------------- |
    /// | 1           |     400     | 8 chars                |
    /// | 2           |     400     | more types of chars    |
    /// | 3           |     400     | 55555                  |
    /// | 4           |     400     | math is hard           |
    /// | 5           |     406     | not joyful enough      |
    /// | 6           |     451     | illegal: no sandwich   |
    /// | 7           |     416     | outranged              |
    /// | 8           |     426     | 😳                     |
    /// | 9           |     418     | not a coffee brewer    |
    /// | None        |     200     | that's a nice password |
    ///
    fn evaluate_complex(&self) -> NaughtyNiceEvaluationResponse {
        match Self::_is_at_least_8_characters_long(&self.input)
            .and_then(Self::_has_uppercase_lowercase_and_digits)
            .and_then(Self::_has_at_least_5_digits)
            .and_then(Self::_integers_add_to_2023)
            .and_then(Self::_is_joyful)
            .and_then(Self::_has_single_spaced_repetition)
            .and_then(Self::_has_at_least_one_unicode_char_between_2980_and_2bff)
            .and_then(Self::_contains_at_least_one_emoji)
            .and_then(Self::_sha256_hash_ends_with_an_a)
        {
            Ok(_) => Ok((
                StatusCode::OK,
                Json(HashMap::from([
                    ("result".to_string(), "nice".to_string()),
                    ("reason".to_string(), "that's a nice password".to_string()),
                ])),
            )),
            Err((status, error)) => Err((
                status,
                Json(HashMap::from([
                    ("result".to_string(), "naughty".to_string()),
                    ("reason".to_string(), error.to_string()),
                ])),
            )),
        }
    }

    /// Verify that the supplied password is at least 8 characters long
    fn _is_at_least_8_characters_long<'input>(
        password: &'input str,
    ) -> ComplexEvaluationResult<'input> {
        if 8 <= password.len() {
            Ok(password)
        } else {
            Err((StatusCode::BAD_REQUEST, "8 chars"))
        }
    }

    /// Verify that the supplied password contains
    /// uppercase letters, lowercase letters, and digits
    fn _has_uppercase_lowercase_and_digits<'input>(
        password: &'input str,
    ) -> ComplexEvaluationResult<'input> {
        if password
            .chars()
            .any(|c| c.is_ascii_digit())
            .bitand(password.chars().any(|c| c.is_uppercase()))
            .bitand(password.chars().any(|c| c.is_lowercase()))
        {
            Ok(password)
        } else {
            Err((StatusCode::BAD_REQUEST, "more types of chars"))
        }
    }

    // Verify that the supplied password contains at least 5 digits
    fn _has_at_least_5_digits<'input>(password: &'input str) -> ComplexEvaluationResult<'input> {
        if 5 <= password.chars().filter(|c| c.is_ascii_digit()).count() {
            Ok(password)
        } else {
            Err((StatusCode::BAD_REQUEST, "55555"))
        }
    }

    // Verify that all integers (that is: sequences of
    // consecutive digits) in the supplied password add
    // up to exactly 2023
    fn _integers_add_to_2023<'input>(password: &'input str) -> ComplexEvaluationResult<'input> {
        if password
            .chars()
            .map(|c| if c.is_ascii_digit() { c } else { ' ' })
            .collect::<String>()
            .split_ascii_whitespace()
            .filter_map(|number| number.parse::<u64>().ok())
            .sum::<u64>()
            == 2023u64
        {
            Ok(password)
        } else {
            Err((StatusCode::BAD_REQUEST, "math is hard"))
        }
    }

    // Verify that the supplied password contains the letters
    // 'j', 'o', and 'y' in that order and in no other order
    fn _is_joyful<'input>(password: &'input str) -> ComplexEvaluationResult<'input> {
        let (mut idx_j, mut idx_o, mut idx_y) = (0usize, 0usize, 0usize);

        for (idx, chr) in password.char_indices() {
            if (0 < idx_o)
                .bitand(0 < idx_y)
                .bitand((idx_o < idx_j).bitor(idx_y < idx_o))
            {
                break;
            }

            match chr {
                'j' => {
                    idx_j = idx;
                }
                'o' => {
                    idx_o = idx;
                }
                'y' => {
                    idx_y = idx;
                }
                _ => {}
            }
        }

        if idx_j < idx_o && idx_o < idx_y {
            Ok(password)
        } else {
            Err((StatusCode::NOT_ACCEPTABLE, "not joyful enough"))
        }
    }

    // Verify that the supplied password contains a letter
    // that repeats with exactly one other letter between
    // repetitions (for example: 'xyx')
    fn _has_single_spaced_repetition<'input>(
        password: &'input str,
    ) -> ComplexEvaluationResult<'input> {
        if password
            .chars()
            .tuple_windows::<(char, char, char)>()
            .any(|(first, second, third)| {
                first == third
                    && first != second
                    && second != third
                    && first.is_ascii_alphabetic()
                    && second.is_ascii_alphabetic()
                    && third.is_ascii_alphabetic()
            })
        {
            Ok(password)
        } else {
            Err((
                StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS,
                "illegal: no sandwich",
            ))
        }
    }

    // Verify that the supplied password contains at least
    // one unicode character in the range (U+2980 - U+2BFF)
    fn _has_at_least_one_unicode_char_between_2980_and_2bff<'input>(
        password: &'input str,
    ) -> ComplexEvaluationResult<'input> {
        if password.chars().any(|c| NICE_RANGE.contains(&c)) {
            Ok(password)
        } else {
            Err((StatusCode::RANGE_NOT_SATISFIABLE, "outranged"))
        }
    }

    // Verify that the supplied password contains at least one emoji
    fn _contains_at_least_one_emoji<'input>(
        password: &'input str,
    ) -> ComplexEvaluationResult<'input> {
        for (idx, chr) in password.char_indices() {
            if EMOJIS.contains(&chr) {
                tracing::warn!(
                    "emoji '{chr}' ({:#x}) detected at position {idx} in password '{password}'",
                    chr as u32
                );
                return Ok(password);
            }
        }

        Err((StatusCode::UPGRADE_REQUIRED, "\u{1F633}"))

        // if password.chars().any(|c| EMOJIS.contains(&c)) {
        //     Ok(password)
        // } else {
        //     Err((StatusCode::UPGRADE_REQUIRED, "\u{1F633}"))
        // }
    }

    // Verify that the hexadecimal representation of the sha256
    // hash of the supplied password ends with an 'a'
    fn _sha256_hash_ends_with_an_a<'input>(
        password: &'input str,
    ) -> ComplexEvaluationResult<'input> {
        if sha256::digest(password)
            .chars()
            .last()
            .is_some_and(|c| c == 'a')
        {
            Ok(password)
        } else {
            Err((StatusCode::IM_A_TEAPOT, "not a coffee brewer"))
        }
    }
}

// </editor-fold desc="// NaughtyNiceEvaluation ...">

/// Complete [Day 15: Task](https://console.shuttle.rs/cch/challenge/15#:~:text=⭐)
#[tracing::instrument(ret, skip(request) fields(error, vowels, input = request.input))]
pub async fn assess_naughty_or_nice(
    Json(request): Json<NaughtyNiceEvaluation>,
) -> NaughtyNiceEvaluationResponse {
    request.evaluate_simple()
}

/// Complete [Day 15: Bonus](https://console.shuttle.rs/cch/challenge/15#:~:text=🎁)
#[allow(unused_variables)]
#[tracing::instrument(ret, skip(request) fields(input = request.input))]
pub async fn game_of_the_year(
    Json(request): Json<NaughtyNiceEvaluation>,
) -> NaughtyNiceEvaluationResponse {
    request.evaluate_complex()
}
