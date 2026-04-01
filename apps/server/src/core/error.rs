use std::collections::HashMap;

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use sqlx::Error as SqlxError;
use utoipa::ToSchema;

const INTERNAL_ERROR_MESSAGE: &str =
    "An internal server error occurred. The error has been reported.";

#[derive(utoipa::IntoResponses)]
pub enum AppError {
    #[response(status = 400, description = "Bad request", example = json!("Json payload is invalid"))]
    BadRequest(String),
    #[response(status = 401, description = "Unauthorized", example = json!("Unauthorized"))]
    Unauthorized(String),
    #[response(status = 404, description = "Not found", example = json!("Project not found"))]
    NotFound(String),
    #[response(status = 409, description = "Conflict")]
    Conflict(ValidationError),
    #[response(status = 422, description = "Unprocessable entity")]
    UnprocessableEntity(ValidationError),
    #[response(status = 500, description = "Unexpected internal server error", example = json!("An internal server error occurred. The error has been reported."))]
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::BadRequest(error) => (StatusCode::BAD_REQUEST, error).into_response(),
            AppError::Unauthorized(error) => (StatusCode::UNAUTHORIZED, error).into_response(),
            AppError::NotFound(error) => (StatusCode::NOT_FOUND, error).into_response(),
            AppError::Conflict(error) => (StatusCode::CONFLICT, Json(error)).into_response(),
            AppError::UnprocessableEntity(error) => {
                (StatusCode::UNPROCESSABLE_ENTITY, Json(error)).into_response()
            }
            AppError::InternalServerError(error) => {
                tracing::error!(error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    INTERNAL_ERROR_MESSAGE.to_string(),
                )
                    .into_response()
            }
        }
    }
}

impl From<SqlxError> for AppError {
    fn from(value: SqlxError) -> Self {
        Self::InternalServerError(format!("SqlxError: {value}"))
    }
}

impl From<axum::extract::rejection::JsonRejection> for AppError {
    fn from(value: axum::extract::rejection::JsonRejection) -> Self {
        Self::BadRequest(value.to_string())
    }
}

impl From<axum::extract::rejection::QueryRejection> for AppError {
    fn from(value: axum::extract::rejection::QueryRejection) -> Self {
        Self::BadRequest(value.to_string())
    }
}

#[derive(Serialize, ToSchema)]
#[serde(untagged)]
pub enum ValidationError {
    #[schema(no_recursion)]
    Map(HashMap<String, ValidationError>),
    List(Vec<ListError>),
    Vec(Vec<String>),
}

#[derive(Serialize, ToSchema)]
pub struct ListError {
    pub index: usize,
    #[schema(no_recursion)]
    pub errors: ValidationError,
}

impl AppError {
    pub fn conflict(field: &str, message: &str) -> Self {
        AppError::Conflict(ValidationError::Map(
            vec![(
                field.to_string(),
                ValidationError::Vec(vec![message.to_string()]),
            )]
            .into_iter()
            .collect(),
        ))
    }
}
