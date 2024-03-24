use miette::Diagnostic;
use thiserror::Error;

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
}
