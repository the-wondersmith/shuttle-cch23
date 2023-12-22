//! ### CCH 2023 Day 21 Solutions
//!

// Standard Library Imports
use core::fmt::Debug;

// Third-Party Imports
use axum::{
    async_trait,
    body::BoxBody,
    extract::{path::Path, FromRef, FromRequestParts},
    http::{request::Parts, Response, StatusCode},
    response::IntoResponse,
};
use dms_coordinates::DMS;
use isocountry::{CountryCode, CountryCodeParseErr};
use s2::{cellid::CellID, latlng::LatLng};
use serde::{Deserialize, Serialize};

// <editor-fold desc="// S2CellId ...">

/// [`axum` extractor](axum::extract) for
/// uploaded tar archive files
#[cfg_attr(test, derive(Eq, Ord, PartialEq, PartialOrd))]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, FromRef)]
pub struct S2CellId(u64);
impl From<S2CellId> for CellID {
    fn from(value: S2CellId) -> Self {
        (&value).into()
    }
}

impl From<&S2CellId> for CellID {
    fn from(value: &S2CellId) -> Self {
        Self(value.0)
    }
}

impl From<S2CellId> for LatLng {
    fn from(value: S2CellId) -> Self {
        (&value).into()
    }
}

impl From<&S2CellId> for LatLng {
    fn from(value: &S2CellId) -> Self {
        <LatLng as From<CellID>>::from(<CellID as From<&S2CellId>>::from(value))
    }
}

#[async_trait]
impl<State: Send + Sync> FromRequestParts<State> for S2CellId {
    type Rejection = Response<BoxBody>;

    #[tracing::instrument(skip_all)]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &State,
    ) -> anyhow::Result<Self, Self::Rejection> {
        <Path<String> as FromRequestParts<State>>::from_request_parts(parts, state)
            .await
            .map_err(IntoResponse::into_response)
            .and_then(|Path(value)| {
                u64::from_str_radix(&value, 2).map_err(|error| {
                    tracing::error!("{:?}", &error);
                    (StatusCode::UNPROCESSABLE_ENTITY, error.to_string()).into_response()
                })
            })
            .map(Self)
    }
}

// </editor-fold desc="// S2CellId ...">

// <editor-fold desc="// GeoAddress ...">

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeoAddress {
    country_code: String,
}

// </editor-fold desc="// GeoAddress ...">

// <editor-fold desc="// GeoCodeResponse ...">

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeoCodeResponse {
    address: GeoAddress,
}

impl GeoCodeResponse {
    fn country(&self) -> Result<CountryCode, (StatusCode, String)> {
        let code = &self.address.country_code.to_uppercase();

        {
            match code.len() {
                2 => CountryCode::for_alpha2(code),
                3 => CountryCode::for_alpha3(code),
                _ => CountryCode::iter()
                    .find(|country| country.name().eq_ignore_ascii_case(code))
                    .ok_or(CountryCodeParseErr::InvalidAlpha3 {
                        unknown: code.to_string(),
                    })
                    .map(|country| *country),
            }
        }
        .map_err(|error| (StatusCode::UNPROCESSABLE_ENTITY, format!("{error:?}")))
    }
}

// </editor-fold desc="// GeoCodeResponse ...">

/// Complete [Day 21: Challenge](https://console.shuttle.rs/cch/challenge/21#:~:text=⭐)
#[tracing::instrument(ret, skip(cell), fields(cell_id = cell.0, lat, lng))]
pub async fn resolve_s2_cell_center(cell: S2CellId) -> impl IntoResponse {
    // Examples:
    //   - "0100111110010011000110011001010101011111000010100011110001011011"
    //     -> 5733954879908101211
    //       -> [83.66508998386551, -30.627939871985497]
    //         -> 83°39'54.324''N 30°37'40.584''W
    //   - "0010000111110000011111100000111010111100000100111101111011000101"
    //     -> 2445593199412240069
    //       -> [-18.915539982809292, 47.5216600194372]
    //         -> 18°54'55.944''S 47°31'17.976''E

    let point: LatLng = cell.into();

    let (lat, lng) = (point.lat.deg(), point.lng.deg());

    tracing::Span::current().record("lat", format!("{lat:.7}"));
    tracing::Span::current().record("lng", format!("{lng:.7}"));

    let (mut lat, mut lng) = (
        DMS::from_decimal_degrees(lat, true),
        DMS::from_decimal_degrees(lng, false),
    );

    lat.seconds = format!("{:.3}", lat.seconds).parse::<f64>().unwrap();
    lng.seconds = format!("{:.3}", lng.seconds).parse::<f64>().unwrap();

    format!("{lat} {lng}")
}

/// Complete [Day 21: Challenge](https://console.shuttle.rs/cch/challenge/21#:~:text=⭐)
#[tracing::instrument(ret, skip(cell), fields(cell_id = cell.0, lat, lng))]
pub async fn resolve_country_from_s2_cell(cell: S2CellId) -> Result<String, (StatusCode, String)> {
    //

    let point: LatLng = cell.into();

    let (lat, lng) = (point.lat.deg(), point.lng.deg());

    tracing::Span::current().record("lat", format!("{lat:.7}"));
    tracing::Span::current().record("lng", format!("{lng:.7}"));

    reqwest::get(format!(
        "https://geocode.maps.co/reverse?lat={lat}&lon={lng}"
    ))
    .await
    .map_err(|error| {
        (
            error.status().unwrap_or(StatusCode::UNPROCESSABLE_ENTITY),
            format!("{error:?}"),
        )
    })?
    .json::<GeoCodeResponse>()
    .await
    .map_err(|error| (StatusCode::UNPROCESSABLE_ENTITY, format!("{error:?}")))?
    .country()
    .map(|country| country.name().replace(" Darussalam", ""))
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
}
