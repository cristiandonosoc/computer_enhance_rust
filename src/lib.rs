#![allow(static_mut_refs)]

pub mod args;
pub mod haversine;
pub mod intel8086;
pub mod json;
pub mod nasm;
pub mod perf;
pub mod utils;

#[macro_use]
extern crate prettytable;

use std::io::{Error, ErrorKind};
use std::path::PathBuf;

pub fn get_cargo_root() -> Result<PathBuf, Error> {
    const ENV_NAME: &str = "CARGO_MANIFEST_DIR";
    let cargo_env = std::env::var(ENV_NAME)
        .map_err(|_| Error::new(ErrorKind::NotFound, format!("env {} not found!", ENV_NAME)))?;
    Ok(PathBuf::from(cargo_env))
}
