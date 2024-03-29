use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenFGAError {
    /*
    {
    "code": "internal_error",
    "message": "Internal Server Error"
    }
    */
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for OpenFGAError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OpenFGAError: {}", self.message)
    }
}

impl std::error::Error for OpenFGAError {}

#[derive(Error, Diagnostic, Debug)]
pub enum Errors {
    #[error(transparent)]
    #[diagnostic(code(openfga::reqwest_error))]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    #[diagnostic(code(openfga::url_parse_error))]
    UrlParseError(#[from] url::ParseError),

    #[error(transparent)]
    #[diagnostic(code(paut::serde_error))]
    SerdeError(#[from] serde_json::Error),

    #[error("missing store id")]
    #[diagnostic(code(openfga::missing_store_id))]
    MissingStoreId,

    #[error("missing model id")]
    #[diagnostic(code(openfga::missing_model_id))]
    MissingModelId,

    #[error("openfga error ")]
    #[diagnostic(code(openfga_error::missing_model_id))]
    OpenFGAErrorRespone(#[from] OpenFGAError),
}
