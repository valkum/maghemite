use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

use ruma_client::Error as RumaError;

/// An error that can occur during client operations.
#[derive(Debug)]
pub struct Error(pub(crate) InnerError);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let message = match self.0 {
            InnerError::AuthenticationRequired => "The queried endpoint requires authentication but was called with an anonymous client.",
            InnerError::Ruma(_) => "An error occurred converting between ruma_client_api and hyper types.",
            InnerError::Channel => "Failed to create a channel",
            InnerError::Tokio => "Tokio library caused an error",
            InnerError::Database => "Datbase error",
            InnerError::Match => "Event did not match the event handlers type"

        };

        write!(f, "{}", message)
    }
}

impl StdError for Error {}

/// Internal representation of errors.
#[derive(Debug)]
pub(crate) enum InnerError {
    /// Queried endpoint requires authentication but was called on an anonymous client.
    AuthenticationRequired,
    /// An error in ruma_client.
    Ruma(RumaError),
    /// An error when creating a channel.
    Channel,
    /// An error in tokio.
    Tokio,
    /// An error in the database.
    Database,
    Match
}

impl From<RumaError> for Error {
    fn from(error: RumaError) -> Self {
        Self(InnerError::Ruma(error))
    }
}