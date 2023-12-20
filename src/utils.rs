//! ## Utilities
//!

// Standard Library Imports
use core::{fmt::Display, ops::Div};
use std::collections::HashMap;

// Third-Party Imports
use axum::http::StatusCode;
use futures::prelude::*;
use image_rs::Pixel;
use serde_json::Value;

// Sub-Module Uses
#[cfg(test)]
#[cfg_attr(test, allow(unused_imports))]
pub(crate) use self::test_utils::{service, TestService};

/// Determine if the supplied value
/// is actually (or effectively) zero
#[inline]
pub fn is_zero<T: Display>(value: T) -> bool {
    value.to_string() == "0"
}

/// Determine if the supplied [`pixel`](image_rs::RGB)
/// would be perceived as "magical red" when viewed with
/// Santa's night vision goggles.
///
/// The goggles considers a pixel "magical red" if the
/// color values of the pixel fulfill the formula:
///
/// > `blue + green < red`
pub fn is_magic_red(data: (u32, u32, image_rs::Rgba<u8>)) -> bool {
    let (_x, _y, rgba) = data;

    let pixel = rgba.to_rgb();

    u16::from(pixel[1]) + u16::from(pixel[2]) < u16::from(pixel[0])
}

/// TODO
#[tracing::instrument(ret)]
pub async fn fetch_pokemon_weight(pokedex_id: u16) -> anyhow::Result<f64, (StatusCode, String)> {
    reqwest::get(format!("https://pokeapi.co/api/v2/pokemon/{pokedex_id}"))
        .map_err(|error| (StatusCode::SERVICE_UNAVAILABLE, error.to_string()))
        .and_then(|response: reqwest::Response| async move {
            if (199u16..300u16).contains(&response.status().as_u16()) {
                response
                    .json::<HashMap<String, Value>>()
                    .await
                    .map_err(|error| (StatusCode::EXPECTATION_FAILED, error.to_string()))
            } else {
                Err((response.status(), format!("{response:?}")))
            }
        })
        .await
        .and_then(|mut data: HashMap<String, Value>| {
            data.remove("weight").ok_or((
                StatusCode::UNPROCESSABLE_ENTITY,
                format!(
                    r#"missing "weight" key from: {}"#,
                    serde_json::to_string(&data).unwrap()
                ),
            ))
        })
        .and_then(|value: Value| value.as_f64().ok_or((StatusCode::NOT_FOUND, String::new())))
        .map(|value| value.div(10f64))
}

#[cfg(test)]
mod test_utils {
    // Standard Library Imports
    use core::fmt::Debug;

    // Third-Party Imports
    use axum::{
        body::{Body, BoxBody},
        http::{
            request::{Builder as RequestBuilder, Request},
            response::Response,
            Method,
        },
        routing::Router as AxumRouter,
    };
    use rstest::fixture;
    use tower::ServiceExt;

    // Crate-Level Imports
    use crate::{router, state::ShuttleAppState};

    const TEST_DB_URL: &str = "postgres://postgres:postgres@localhost:19867/postgres";

    // <editor-fold desc="// TestService ...">
    #[derive(Debug)]
    pub(crate) struct TestService(AxumRouter);

    impl Default for TestService {
        fn default() -> Self {
            let db = sqlx::PgPool::connect_lazy(TEST_DB_URL).unwrap();
            let state = ShuttleAppState::initialize(db, None, None, None).unwrap();

            Self(router(state))
        }
    }

    impl TestService {
        /// Bounce the supplied request body off the project's
        /// `axum::Router` at the specified path and return the
        /// resolved response
        pub(crate) async fn resolve<Resolvable: TryIntoRequest<Body>>(
            self,
            request: Resolvable,
        ) -> anyhow::Result<Response<BoxBody>> {
            let request = request.into_request()?;

            Ok(self.0.oneshot(request).await?)
        }
    }

    // </editor-fold desc="// TestService ...">

    // <editor-fold desc="// Fixtures ...">

    #[fixture]
    pub(crate) fn service() -> TestService {
        TestService::default()
    }

    // </editor-fold desc="// Fixtures ...">

    // <editor-fold desc="// Helper Traits ...">

    pub(crate) trait IntoBody {
        fn into_body(self) -> Body;
    }

    pub(crate) trait IntoMethod {
        fn into_method(self) -> Method;
    }

    pub(crate) trait TryIntoRequest<T: Debug> {
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

    impl TryIntoRequest<Body> for RequestBuilder {
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

    // </editor-fold desc="// Helper Traits ...">
}
