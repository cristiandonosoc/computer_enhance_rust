pub mod error;
pub mod simulation;

use computer_enhance_rust::{get_cargo_root, nasm::*};
use error::TestError;
use similar::{ChangeTag, TextDiff};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use computer_enhance_rust::intel8086;

pub fn run_nasm_test(listing_name: &str) -> Result<(), TestError> {
    // Create a temporary dir for this test.
    let temp_dir = TempDir::new().map_err(|e| TestError::io(String::from("TempDir"), e))?;

    // Run nasm on the input file.
    let listing = find_listing(listing_name)?;

    let want_bytes = run_nasm(temp_dir.path(), &listing)
        .map_err(|e| TestError::io(listing.display().to_string(), e))?;
    println!("WANT BYTES: {:02X?}", want_bytes);

    let got_instructions = intel8086::disassemble(&want_bytes)?;
    let got_asm = intel8086::to_asm(&got_instructions);

    // Write the asm we got into a file.
    let temp_asm_file = temp_dir.path().join("test.asm");
    std::fs::write(&temp_asm_file, &got_asm)
        .map_err(|e| TestError::io(temp_asm_file.display().to_string(), e))?;

    // Read the asm file and they should be the same.
    let got_bytes = run_nasm(temp_dir.path(), temp_asm_file)
        .map_err(|e| TestError::io("nasm".to_string(), e))?;
    println!(" GOT BYTES: {:02X?}", got_bytes);

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

fn find_listing(listing: &str) -> Result<PathBuf, TestError> {
    let mut path = get_cargo_root().map_err(|e| TestError::io("cargo".to_string(), e))?;
    path = path.join("extras/listings").join(listing);
    if !path.exists() {
        return Err(TestError::not_found(path.display().to_string()));
    }
    Ok(path)
}

// Poor man's clean: Cleaning a file means removing empty lines and comments.
// That way we can diff only on the actual content.
fn clean_asm_file(input: &str) -> String {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with(';'))
        .collect::<Vec<&str>>()
        .join("\n")
}
