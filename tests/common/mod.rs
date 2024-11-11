use similar::{ChangeTag, TextDiff};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::{NamedTempFile, TempDir};
use thiserror::Error;

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
    IntelError(#[from] intel8086::IntelError),
}

impl TestError {
    fn not_found(element: String) -> TestError {
        TestError::IO {
            element,
            err: std::io::Error::new(ErrorKind::NotFound, ""),
        }
    }

    fn io(element: String, source: std::io::Error) -> TestError {
        TestError::IO {
            element,
            err: source,
        }
    }
}

use computer_enhance_rust::intel8086;

pub fn run_nasm_test(listing_name: &str) -> Result<(), TestError> {
    // Create a temporary dir for this test.
    let temp_dir = TempDir::new().map_err(|e| TestError::io(String::from("TempDir"), e))?;

    // Run nasm on the input file.
    let listing = find_listing(listing_name)?;

    let want_bytes = run_nasm(temp_dir.path(), &listing)?;
    println!("WANT BYTES: {:02X?}", want_bytes);

    let got_instructions = intel8086::disassemble(&want_bytes)?;
    let got_asm = intel8086::to_asm(&got_instructions);

    // return Err(Box::new(Error::new(ErrorKind::Other, "TEST FAILED")));

    // Write the asm we got into a file.
    let temp_asm_file = temp_dir.path().join("test.asm");
    std::fs::write(&temp_asm_file, &got_asm)
        .map_err(|e| TestError::io(temp_asm_file.display().to_string(), e))?;

    // Read the asm file and they should be the same.
    let got_bytes = run_nasm(temp_dir.path(), temp_asm_file)?;
    println!("GOT BYTES: {:02X?}", got_bytes);

    if want_bytes == got_bytes {
        return Ok(());
    }

    // If they are not equal, we compare the input listing file with our asm.
    let listing_str = std::fs::read_to_string(&listing)
        .map_err(|e| TestError::io(listing.display().to_string(), e))?;
    let clean_want = clean_asm_file(listing_str.as_str());
    let clean_got = clean_asm_file(got_asm.as_str());

    println!("DIFF:");
    let diff = TextDiff::from_lines(&clean_want, &clean_got);
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => print!("-{}", change),
            ChangeTag::Insert => print!("+{}", change),
            ChangeTag::Equal => print!(" {}", change),
        }
    }

    assert!(false);
    return Ok(());
}

fn run_nasm(output_dir: &Path, filepath: impl AsRef<Path>) -> Result<Vec<u8>, TestError> {
    let cargo_root = get_cargo_root()?;
    let nasm = cargo_root.join("extras").join("nasm").join("nasm.exe");

    let temp_file = NamedTempFile::new_in(output_dir)
        .map_err(|e| TestError::io(output_dir.display().to_string(), e))?;

    let _ = Command::new(nasm)
        .args(["-o", temp_file.path().to_str().unwrap()])
        .arg(filepath.as_ref().as_os_str())
        .output()
        .map_err(|e| TestError::io(filepath.as_ref().display().to_string(), e))?;

    let bytes = std::fs::read(&temp_file)
        .map_err(|e| TestError::io(temp_file.path().display().to_string(), e))?;
    Ok(bytes)
}

fn find_listing(listing: &str) -> Result<PathBuf, TestError> {
    let mut path = get_cargo_root()?;
    path = path.join("extras/listings").join(listing);
    if !path.exists() {
        return Err(TestError::not_found(path.display().to_string()));
    }
    Ok(path)
}

fn get_cargo_root() -> Result<PathBuf, TestError> {
    const ENV_NAME: &str = "CARGO_MANIFEST_DIR";
    let cargo_env = std::env::var(ENV_NAME).map_err(|_| TestError::EnvNotFound {
        env: ENV_NAME.to_string(),
    })?;
    Ok(PathBuf::from(cargo_env))
}

fn clean_asm_file(input: &str) -> String {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with(';'))
        .collect::<Vec<&str>>()
        .join("\n")
}