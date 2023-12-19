//! ## Custom Types
//!

// Standard Library Imports
use core::fmt::Debug;
use std::{
    boxed::Box,
    collections::BTreeMap,
    env::{set_var as set_env_var, var as get_env_var},
    path::PathBuf as FilePathBuf,
};

// Third-Party Imports
use axum::extract::FromRef;
#[allow(unused_imports)]
use axum_template::{
    engine::{Engine as HandlebarsEngine, HandlebarsError},
    Key, RenderHtml,
};
use handlebars::{Handlebars, TemplateError};

use shuttle_persist::{PersistError as PersistenceError, PersistInstance as Persistence};
use shuttle_secrets::SecretStore;

pub(super) type TemplateEngine = HandlebarsEngine<Handlebars<'static>>;

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
            .map(FilePathBuf::from)
            .unwrap_or(
                FilePathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join(".shuttle-storage")
                    .join("shuttle-persist"),
            );

        Persistence::new(path)
    }
}

// </editor-fold desc="// ShuttleAppState ...">
