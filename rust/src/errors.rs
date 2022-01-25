//! Crate-specific errors (using `failure`)
//!
//! The justification for this complexity is that we need to return correct HTTP error types as
//! well as helpful error messages in API responses.

// Design goals:
// - be able to call '?' on random things and collect them in endpoint handlers
// - be able to 'bail!()' and 'ensure!()' and have those end up as InternalErrors
// - do conversion into ErrorResponse model in this file, not endpoint handlers, and ErrorReponse
//   should have good context about the error
//
// Plan:
// - use .map_err() to convert to the correct type
// - map to

pub use failure::Error;
use failure::Fail;
use fatcat_openapi::models;
use std::result;

/// A type alias for handling errors throughout this crate
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Fail, Clone)]
pub enum FatcatError {
    #[fail(display = "no such {} found: {}", _0, _1)]
    NotFound(String, String),

    #[fail(
        display = "invalid fatcat identifier (expect 26-char base32 encoded): {}",
        _0
    )]
    InvalidFatcatId(String),

    #[fail(
        display = "external identifier doesn't match required pattern for a {}: {}",
        _0, _1
    )]
    MalformedExternalId(String, String),

    #[fail(
        display = "checksum doesn't match required pattern ({} in hex encoding): {}",
        _0, _1
    )]
    MalformedChecksum(String, String),

    #[fail(display = "not a valid UUID: {}", _0)]
    MalformedUuid(String),

    #[fail(display = "'{}' is not a an known/acceptable '{}' value", _1, _0)]
    NotInControlledVocabulary(String, String), // vocab, word

    #[fail(
        display = "attempted to accept or mutate an editgroup which was already accepted: {}",
        _0
    )]
    EditgroupAlreadyAccepted(String),

    #[fail(
        display = "external identifiers missing or multiple specified; please supply exactly one: {}",
        _0
    )]
    MissingOrMultipleExternalId(String),

    #[fail(display = "tried to mutate an entity into impossible state: {}", _0)]
    InvalidEntityStateTransform(String),

    #[fail(
        display = "auth token was missing, expired, revoked, or corrupt: {}",
        _0
    )]
    InvalidCredentials(String),

    #[fail(display = "editor account doesn't have authorization: {}", _0)]
    InsufficientPrivileges(String),

    #[fail(
        display = "broke a constraint or made an otherwise invalid request: {}",
        _0
    )]
    // Utf8Decode, StringDecode, Uuid
    BadRequest(String),

    #[fail(display = "database error: {}", _0)]
    // Diesel constraint that we think is a user error
    ConstraintViolation(String),

    #[fail(display = "database in read-only mode (usually replica or maintenance)")]
    DatabaseReadOnly,

    #[fail(display = "generic database 'not-found'")]
    // This should generally get caught and handled
    DatabaseRowNotFound,

    // TODO: can these hold context instead of Inner?
    #[fail(display = "unexpected database error: {}", _0)]
    // other Diesel, R2d2 errors which we don't think are user errors (eg, connection failure)
    DatabaseError(String),

    #[fail(display = "unexpected internal error: {}", _0)]
    // Fmt, Io, Serde,
    InternalError(String),
} // NOTE: this enum is not exhaustive and shouldn't be matched over!

impl Into<models::ErrorResponse> for FatcatError {
    /// Format an error as an API response (ErrorResponse model, used by all HTTP 4xx and 5xx
    /// responses)
    fn into(self) -> models::ErrorResponse {
        // TODO: something more complex? context?
        models::ErrorResponse {
            success: false,
            // enum variant name, without fields. whew, what a pile
            error: format!("{:?}", self).split('(').collect::<Vec<&str>>()[0].to_string(),
            message: self.to_string(),
        }
    }
}

impl From<diesel::result::Error> for FatcatError {
    fn from(inner: diesel::result::Error) -> FatcatError {
        match inner {
            diesel::result::Error::NotFound => FatcatError::DatabaseRowNotFound,
            diesel::result::Error::DatabaseError(_, info)
                if info.message().contains("in a read-only transaction") =>
            {
                FatcatError::DatabaseReadOnly
            }
            diesel::result::Error::DatabaseError(_, _) => {
                FatcatError::ConstraintViolation(inner.to_string())
            }
            _ => FatcatError::InternalError(inner.to_string()),
        }
    }
}

impl From<std::fmt::Error> for FatcatError {
    fn from(inner: std::fmt::Error) -> FatcatError {
        FatcatError::InternalError(inner.to_string())
    }
}

impl From<diesel::r2d2::Error> for FatcatError {
    fn from(inner: diesel::r2d2::Error) -> FatcatError {
        FatcatError::InternalError(inner.to_string())
    }
}

impl From<uuid::ParseError> for FatcatError {
    fn from(inner: uuid::ParseError) -> FatcatError {
        FatcatError::MalformedUuid(inner.to_string())
    }
}

impl From<serde_json::Error> for FatcatError {
    fn from(inner: serde_json::Error) -> FatcatError {
        FatcatError::InternalError(inner.to_string())
    }
}

impl From<std::string::FromUtf8Error> for FatcatError {
    fn from(inner: std::string::FromUtf8Error) -> FatcatError {
        FatcatError::InternalError(inner.to_string())
    }
}

impl From<data_encoding::DecodeError> for FatcatError {
    fn from(inner: data_encoding::DecodeError) -> FatcatError {
        FatcatError::InternalError(inner.to_string())
    }
}

// A big catchall!
impl From<failure::Error> for FatcatError {
    fn from(error: failure::Error) -> FatcatError {
        // TODO: I think it should be possible to match here? regardless, this is *super* janky
        if let Some(_) = error.downcast_ref::<FatcatError>() {
            return error.downcast::<FatcatError>().unwrap();
        }
        if let Some(_) = error.downcast_ref::<std::fmt::Error>() {
            return error.downcast::<std::fmt::Error>().unwrap().into();
        }
        if let Some(_) = error.downcast_ref::<diesel::result::Error>() {
            return error.downcast::<diesel::result::Error>().unwrap().into();
        }
        if let Some(_) = error.downcast_ref::<uuid::ParseError>() {
            return error.downcast::<uuid::ParseError>().unwrap().into();
        }
        FatcatError::InternalError(error.to_string())
    }
}
