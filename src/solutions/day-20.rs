//! ### CCH 2023 Day 20 Solutions
//!

// Standard Library Imports
use core::{
    error::Error as GenericError,
    fmt::{Debug, Formatter, Result as FormatResult},
    ops::{BitOr, Deref, DerefMut, Not},
};

// Third-Party Imports
use axum::{
    async_trait,
    body::Bytes,
    extract::{FromRequest, FromRequestParts, Json, TypedHeader},
    headers::ContentType,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use bytes::{buf::Reader as ByteReader, Buf};
use git2::Repository as GitRepo;
use num_traits::FromPrimitive;
use once_cell::sync::Lazy;

// <editor-fold desc="// Utilities ...">

fn as_412_response<E: GenericError>(error: E) -> Response {
    tracing::error!("{:?}", &error);
    (StatusCode::PRECONDITION_FAILED, error.to_string()).into_response()
}

// </editor-fold desc="// Utilities ...">

// <editor-fold desc="// UploadedTarArchive ...">

/// [`axum` extractor](axum::extract) for
/// uploaded tar archive files
pub struct UploadedTarArchive(tar::Archive<ByteReader<Bytes>>, usize);

#[allow(clippy::declare_interior_mutable_const)]
impl UploadedTarArchive {
    const MIME: Lazy<ContentType> =
        Lazy::new(|| "application/x-tar".parse::<ContentType>().unwrap());
}

impl Deref for UploadedTarArchive {
    type Target = tar::Archive<ByteReader<Bytes>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UploadedTarArchive {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Debug for UploadedTarArchive {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        write!(formatter, "UploadedTarArchive({} bytes)", self.1)
    }
}

#[async_trait]
impl<State: Send + Sync, BodyType: Send + 'static> FromRequest<State, BodyType>
    for UploadedTarArchive
where
    Bytes: FromRequest<State, BodyType>,
{
    type Rejection = Response;

    #[allow(clippy::borrow_interior_mutable_const)]
    #[tracing::instrument(err(Debug), skip_all)]
    async fn from_request(
        request: Request<BodyType>,
        state: &State,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = request.into_parts();

        let content_type =
            <Option<TypedHeader<ContentType>> as FromRequestParts<State>>::from_request_parts(
                &mut parts, state,
            )
            .await
            .map(|value| value.map(|header| header.0))
            .map_err(IntoResponse::into_response)?;

        if content_type
            .as_ref()
            .is_some_and(|value| value != Self::MIME.deref())
        {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("unexpected content type: {content_type:?}"),
            )
                .into_response());
        }

        let request = Request::<BodyType>::from_parts(parts, body);

        Bytes::from_request(request, state)
            .await
            .map(|body| {
                let size = body.len();
                Self(tar::Archive::new(body.reader()), size)
            })
            .map_err(IntoResponse::into_response)
    }
}

// </editor-fold desc="// UploadedTarArchive ...">

/// Endpoint 1/2 for [Day 20: Task](https://console.shuttle.rs/cch/challenge/20#:~:text=⭐️)
#[tracing::instrument(ret, err(Debug), skip_all)]
pub async fn get_archived_file_count(
    mut archive: UploadedTarArchive,
) -> Result<Json<u64>, (StatusCode, String)> {
    archive
        .entries()
        .map_err(|error| (StatusCode::UNPROCESSABLE_ENTITY, error.to_string()))
        .and_then(|entries| {
            let file_count = entries.count();

            u64::from_usize(file_count).map(Json).ok_or_else(|| {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!("error casting {file_count} as u64"),
                )
            })
        })
}

/// Endpoint 2/2 for [Day 20: Task](https://console.shuttle.rs/cch/challenge/20#:~:text=⭐️)
#[tracing::instrument(ret, err(Debug), skip_all)]
pub async fn get_total_archived_file_size(
    mut archive: UploadedTarArchive,
) -> Result<Json<u64>, (StatusCode, String)> {
    let entries = archive
        .entries()
        .map_err(|error| (StatusCode::UNPROCESSABLE_ENTITY, error.to_string()))?;

    let mut total = 0u64;
    let file_count = entries.size_hint().1.unwrap_or(usize::MAX);

    for (idx, entry) in entries.enumerate() {
        match entry {
            Ok(entry) => {
                total += entry.size();
            }
            Err(error) => {
                return Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!("could not evaluate size of {idx}/{file_count}: {error:?}"),
                ));
            }
        }
    }

    Ok(Json(total))
}

/// Complete [Day 20: Bonus](https://console.shuttle.rs/cch/challenge/20#:~:text=🎁️)
///
/// > **NOTE:** I hate this fucking function so god damn much.
/// >           The only thing I learned from writing it is that
/// >           git is fantastic, but things like how it works
/// >           under the hood or how to traverse its structure(s)
/// >           are absolutely none of my fucking business.
#[tracing::instrument(ret, err(Debug), skip_all)]
pub async fn git_blame_cookie_hunt(
    UploadedTarArchive(mut archive, _): UploadedTarArchive,
) -> Result<String, Response> {
    let temp = tempfile::tempdir().map_err(as_412_response)?;

    archive.unpack(temp.path()).map_err(as_412_response)?;

    let repo = GitRepo::open(temp.path()).map_err(as_412_response)?;

    let branch = repo
        .find_branch("christmas", git2::BranchType::Local)
        .map_err(as_412_response)?;

    let tree = branch.get().peel_to_tree().map_err(as_412_response)?;

    let mut options = git2::build::CheckoutBuilder::new();

    repo.checkout_tree(tree.as_object(), Some(options.force()))
        .map_err(as_412_response)?;

    repo.set_head(branch.get().name().unwrap())
        .map_err(as_412_response)?;

    let mut walker = repo.revwalk().map_err(as_412_response)?;
    walker.push_head().map_err(as_412_response)?;

    let mut cookie_commit: Option<(String, git2::Oid)> = None;

    for id in walker.filter(Result::is_ok).map(Result::unwrap) {
        let commit = repo.find_commit(id).map_err(as_412_response)?;

        let commit_id = commit
            .parent_ids()
            .next()
            .and_then(|pid| repo.find_tree(pid).ok());

        let mut diff = repo
            .diff_tree_to_tree(
                Some(&commit.tree().map_err(as_412_response)?),
                commit_id.as_ref(),
                Some(&mut git2::DiffOptions::new()),
            )
            .map_err(as_412_response)?;

        diff.find_similar(Some(
            git2::DiffFindOptions::new()
                .copies(false)
                .break_rewrites(false),
        ))
        .map_err(as_412_response)?;

        for file in diff
            .deltas()
            .filter(|delta| {
                delta
                    .old_file()
                    .path()
                    .and_then(|path| path.file_name())
                    .is_some_and(|name| name == "santa.txt")
                    .bitor(
                        delta
                            .new_file()
                            .path()
                            .and_then(|path| path.file_name())
                            .is_some_and(|name| name == "santa.txt"),
                    )
            })
            .filter_map(|delta| {
                let (new_file, old_file) = (delta.new_file(), delta.old_file());

                match (old_file.id().is_zero().not(), new_file.id().is_zero().not()) {
                    (true, false) => Some(old_file),
                    (false, true) | (true, true) => Some(new_file),
                    (false, false) => None,
                }
            })
        {
            if let Ok(blob) = repo
                .find_object(file.id(), Some(git2::ObjectType::Blob))
                .map_err(as_412_response)?
                .into_blob()
            {
                let contents = String::from_utf8_lossy(blob.content()).to_string();

                if contents.to_uppercase().contains("COOKIE") {
                    cookie_commit = Some((
                        String::from_utf8_lossy(commit.author().name_bytes()).to_string(),
                        commit.id(),
                    ));
                    break;
                }
            }
        }

        if cookie_commit.is_some() {
            break;
        }
    }

    match cookie_commit {
        Some((author, commit)) => Ok(format!("{author} {commit}")),
        None => Err((StatusCode::NOT_FOUND, "cookie commit not found".to_string()).into_response()),
    }
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
