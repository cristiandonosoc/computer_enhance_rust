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
    #[error("IntelError: {0}")]
    IntelError(#[from] IntelError),

    #[error("Wrong test result")]
    WrongResult,

    #[error("Custom Error: {0}")]
    CustomError(String)
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

    pub(super) fn custom(msg: String) -> TestError {
        TestError::CustomError {
            0: msg,
        }
    }
}




