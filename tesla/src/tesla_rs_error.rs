use std::fmt;
use std::error;

use reqwest;

#[derive(Debug)]
pub struct AppError {
    pub(crate) message: String,
}

impl error::Error for AppError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Application error : {}", self.message)
    }
}


#[derive(Debug)]
pub enum TeslaError {
    ParseAppError(AppError),
    AuthError,
    InvalidTokenError,
    ParseReqwest(reqwest::Error)
}

impl error::Error for TeslaError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            TeslaError::ParseAppError(ref e) => Some(e),
            TeslaError::AuthError => None,
            TeslaError::InvalidTokenError => None,
            TeslaError::ParseReqwest(ref e) => Some(e),
        }
    }
}

impl fmt::Display for TeslaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TeslaError::ParseAppError(ref e) => e.fmt(f),
            TeslaError::AuthError => write!(f, "Authentication error!"),
            TeslaError::InvalidTokenError => write!(f, "Invalid token error!"),
            TeslaError::ParseReqwest(ref e) => e.fmt(f),
        }
    }
}

impl From<AppError> for TeslaError {
    fn from(err: AppError) -> TeslaError {
        TeslaError::ParseAppError(err)
    }
}

impl From<reqwest::Error> for TeslaError {
    fn from(err: reqwest::Error) -> TeslaError {
        TeslaError::ParseReqwest(err)
    }
}
