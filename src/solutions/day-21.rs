#![allow(unused_imports)]
//! ### CCH 2023 Day 21 Solutions
//!

// Standard Library Imports
use core::ops::{Add, BitAnd, BitXor, Sub};
use std::collections::HashMap;
use std::ops::BitOr;

// Third-Party Imports
use axum::{
    body::Body,
    extract::{multipart::Multipart, Json, Path, State},
    http::{Request, StatusCode},
    response::IntoResponse,
    routing,
};
use axum_template::TemplateEngine;
use chrono::{DateTime, Datelike, Utc};
use futures::prelude::*;
use image_rs::GenericImageView;
use itertools::Itertools;
use serde_json::{Map as JsonObject, Value};
use shuttle_persist::{Persist, PersistInstance as Persistence};
use shuttle_secrets::{SecretStore, Secrets};
use shuttle_shared_db::Postgres as PgDb;
use tower::ServiceExt;
use tower_http::services::ServeFile;
use unicode_normalization::UnicodeNormalization;

// Crate-Level Imports
use crate::{types, utils};
