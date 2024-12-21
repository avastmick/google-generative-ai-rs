use reqwest::StatusCode;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct GoogleAPIError {
    pub message: String,
    pub code: Option<StatusCode>,
}
impl fmt::Display for GoogleAPIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "GoogleAPIError - code: {:?} error: {}",
            self.code, self.message
        )
    }
}
impl Error for GoogleAPIError {}
