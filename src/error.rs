use crate::client::ErrorReceipt;
use http::StatusCode;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{reason}, was `{actual}`")]
    Url { actual: Url, reason: String },

    #[error(transparent)]
    HttpClient(#[from] reqwest::Error),

    #[error(transparent)]
    Postmark(#[from] PostmarkError),
}

#[derive(Error, Debug)]
pub enum PostmarkError {
    /// [Postmark's documentation](https://postmarkapp.com/developer/api/overview#error-codes).
    #[error("received response with error `{0}`")]
    UnprocessableEntity(ErrorReceipt),

    /// [Postmark's documentation](https://postmarkapp.com/developer/api/overview#response-codes).
    #[error("received response with status code `{0}`")]
    Other(StatusCode),
}
