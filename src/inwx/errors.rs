extern crate reqwest;

use std::fmt;
use std::error;
use std;

pub type UpdateResult = std::result::Result<(), UpdateError>;

#[derive(Debug)]
pub enum UpdateError {
    UpdateFailed{code: u32, message: String},
    Network(reqwest::Error),
    UnexpectedAnswer(String),
}

impl error::Error for UpdateError {
    fn description(&self) -> &str {
        match *self {
            UpdateError::UpdateFailed {..} => "update was rejected",
            UpdateError::Network(_) => "a network error occured",
            UpdateError::UnexpectedAnswer(_) => "server sent unexpected answer",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &UpdateError::Network(ref err) => Some(&*err as &error::Error),
            _ => None,
        }
    }
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UpdateError::UpdateFailed {code, ref message} => write!(f, "Server sent error code {}: {}", code, message),
            UpdateError::UnexpectedAnswer(ref s) => write!(f, "Unexpected answer from server: {}", s),
            UpdateError::Network(ref err) => write!(f, "Network error: {}", err),
        }
    }
}

impl std::convert::From<reqwest::Error> for UpdateError {
    fn from(err: reqwest::Error) -> UpdateError {
        UpdateError::Network(err)
    }
}


