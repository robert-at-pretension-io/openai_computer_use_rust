//! Error types for the CUA Desktop implementation

use std::fmt;
use std::error::Error;

/// Custom error type for CUA operations
#[derive(Debug)]
pub enum CuaError {
    /// Error related to computer actions (mouse, keyboard)
    ActionError(String),
    
    /// Error related to screenshots
    ScreenshotError(String),
    
    /// Error related to API calls
    ApiError(String),
    
    /// Error related to safety checks
    SafetyError(String),
    
    /// IO error from standard library
    IoError(std::io::Error),
    
    /// General error
    Other(String),
}

impl fmt::Display for CuaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CuaError::ActionError(msg) => write!(f, "Action error: {}", msg),
            CuaError::ScreenshotError(msg) => write!(f, "Screenshot error: {}", msg),
            CuaError::ApiError(msg) => write!(f, "API error: {}", msg),
            CuaError::SafetyError(msg) => write!(f, "Safety error: {}", msg),
            CuaError::IoError(err) => write!(f, "IO error: {}", err),
            CuaError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl Error for CuaError {}

impl From<std::io::Error> for CuaError {
    fn from(err: std::io::Error) -> Self {
        CuaError::IoError(err)
    }
}

// Implement From for hyper errors
impl From<hyper::Error> for CuaError {
    fn from(err: hyper::Error) -> Self {
        CuaError::ApiError(format!("HTTP error: {}", err))
    }
}

// Implement From for http errors
impl From<http::Error> for CuaError {
    fn from(err: http::Error) -> Self {
        CuaError::ApiError(format!("HTTP error: {}", err))
    }
}

// Implement From for serde_json errors
impl From<serde_json::Error> for CuaError {
    fn from(err: serde_json::Error) -> Self {
        CuaError::ApiError(format!("JSON error: {}", err))
    }
}