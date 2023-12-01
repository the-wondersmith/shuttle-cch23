#![allow(clippy::unit_arg)]
//! ## Integration Tests For `cch23-thewondersmith`

// Module Declarations

// Standard Library Imports

// Third Part Imports
use pretty_assertions::assert_str_eq;
use rstest::rstest;

// Crate-Level Imports

// <editor-fold desc="// Constants ...">

// </editor-fold desc="// Constants ...">

// <editor-fold desc="// Fixtures ...">

// </editor-fold desc="// Fixtures ...">

// <editor-fold desc="// Utility Functions ...">

// </editor-fold desc="// Utility Functions ...">

// <editor-fold desc="// Integration Tests ...">

#[rstest]
#[test_log::test(tokio::test)]
/// TODO(the-wondersmith): DOCUMENTATION
async fn shuttle_is_dope() -> anyhow::Result<()> {
    Ok(assert_str_eq!("GOT EM", "got em".to_ascii_uppercase()))
}

// </editor-fold desc="// Integration Tests ...">
