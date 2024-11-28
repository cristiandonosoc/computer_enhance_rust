use computer_enhance_rust::intel8086::error::IntelError;
use thiserror::Error;
use std::io::ErrorKind;

#[derive(Error, Debug)]
pub enum TestError {
    #[error("IOError: {element}: {err}")]
    IO {
        element: String,
        err: std::io::Error,
    },
    #[error("Environment variable {env} not found")]
    EnvNotFound { env: String },
    #[error("IntelError: {0}")]
    IntelError(#[from] IntelError),
    #[error("Program error: {stderr}\nContent:\n{content}")]
    NasmError { stderr: String, content: String },
}

impl TestError {
    pub(super) fn not_found(element: String) -> TestError {
        TestError::IO {
            element,
            err: std::io::Error::new(ErrorKind::NotFound, ""),
        }
    }

    pub(super) fn io(element: String, source: std::io::Error) -> TestError {
        TestError::IO {
            element,
            err: source,
        }
    }
}




