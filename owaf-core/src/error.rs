use salvo::http::{ParseError, StatusCode, StatusError};
use salvo::oapi::{self, EndpointOutRegister, ToSchema};
use salvo::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("public: `{0}`")]
    Public(String),
    #[error("internal: `{0}`")]
    Internal(String),
    #[error("salvo internal error: `{0}`")]
    Salvo(#[from] ::salvo::Error),
    #[error("http status error: `{0}`")]
    HttpStatus(#[from] StatusError),
    #[error("http parse error:`{0}`")]
    HttpParse(#[from] ParseError),
    #[error("anyhow error:`{0}`")]
    Anyhow(#[from] anyhow::Error),
    #[error("sqlx::Error:`{0}`")]
    SqlxError(#[from] sqlx::Error),
    #[error("validation error:`{0}`")]
    Validation(#[from] validator::ValidationErrors),
}
impl AppError {
    pub fn public<S: Into<String>>(msg: S) -> Self {
        Self::Public(msg.into())
    }

    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }
}

#[async_trait]
impl Writer for AppError {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let code = match &self {
            Self::HttpStatus(e) => e.code,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        res.status_code(code);
        let data = match self {
            Self::Salvo(e) => {
                tracing::error!(error = ?e, "salvo error");
                StatusError::internal_server_error().brief("Unknown error happened in salvo.")
            }
            Self::Public(msg) => StatusError::internal_server_error().brief(msg),
            Self::Internal(msg) => {
                tracing::error!(msg = msg, "internal error");
                StatusError::internal_server_error()
            }
            Self::HttpStatus(e) => e,
            e => StatusError::internal_server_error()
                .brief(format!("Unknown error happened: {e}"))
                .cause(e),
        };
        res.render(data);
    }
}
impl EndpointOutRegister for AppError {
    fn register(components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        operation.responses.insert(
            StatusCode::INTERNAL_SERVER_ERROR.as_str(),
            oapi::Response::new("Internal server error")
                .add_content("application/json", StatusError::to_schema(components)),
        );
        operation.responses.insert(
            StatusCode::NOT_FOUND.as_str(),
            oapi::Response::new("Not found")
                .add_content("application/json", StatusError::to_schema(components)),
        );
        operation.responses.insert(
            StatusCode::BAD_REQUEST.as_str(),
            oapi::Response::new("Bad request")
                .add_content("application/json", StatusError::to_schema(components)),
        );
    }
}
