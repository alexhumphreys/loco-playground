use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum Errors {
    #[error(transparent)]
    #[diagnostic(code(openfga::reqwest_error))]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    #[diagnostic(code(paut::serde_error))]
    SerdeError(#[from] serde_json::Error),
}
