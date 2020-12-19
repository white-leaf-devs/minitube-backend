use netlify_lambda_http::http::StatusCode;
use netlify_lambda_http::{Body, IntoResponse, Response};
use rusoto_core::RusotoError;
use serde_json::{self as json, json};
use std::error::Error as StdError;
use std::fmt::Display;
use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid route: {route}")]
    InvalidRoute { route: String },

    #[error("Rusoto error: {info}")]
    RusotoError { info: String },

    #[error("Invalid request: {info}")]
    InvalidRequest { info: String },

    #[error("Internal error: {info}")]
    InternalError { info: String },

    #[error("I/O error: {source}")]
    IoError {
        #[from]
        source: io::Error,
    },

    #[error("UTF-8 error: {source}")]
    Utf8Error {
        #[from]
        source: FromUtf8Error,
    },

    #[error("JSON error: {source}")]
    JsonError {
        #[from]
        source: json::Error,
    },

    #[error("Base64 decoder error: {source}")]
    Base64DecoderError {
        #[from]
        source: base64::DecodeError,
    },

    #[error("AWS credentials error: {source}")]
    CredentialsError {
        #[from]
        source: rusoto_credential::CredentialsError,
    },
}

impl IntoResponse for Error {
    fn into_response(self) -> Response<Body> {
        let (code, value) = match self {
            Self::InvalidRoute { route } => {
                let json = json! {{
                    "error_type": "invalid_route",
                    "info": format!("route={}", route)
                }};

                (StatusCode::NOT_FOUND, json)
            }

            Self::InvalidRequest { info } => {
                let json = json! {{
                    "error_type": "invalid_request",
                    "info": info
                }};

                (StatusCode::BAD_REQUEST, json)
            }

            _ => {
                let json = json! {{
                    "error_type": "internal_error",
                    "info": self.to_string()
                }};

                (StatusCode::INTERNAL_SERVER_ERROR, json)
            }
        };

        Response::builder()
            .status(code)
            .body(Body::Text(value.to_string()))
            .unwrap()
    }
}

impl<E: Display + StdError + 'static> From<RusotoError<E>> for Error {
    fn from(err: RusotoError<E>) -> Self {
        Self::RusotoError {
            info: format!("{}", err),
        }
    }
}

impl Error {
    pub fn invalid_route<S: Into<String>>(route: S) -> Self {
        Self::InvalidRoute {
            route: route.into(),
        }
    }

    pub fn invalid_request<S: Into<String>>(info: S) -> Self {
        Self::InvalidRequest { info: info.into() }
    }

    pub fn internal_error<S: Into<String>>(info: S) -> Self {
        Self::InternalError { info: info.into() }
    }
}
