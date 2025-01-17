use super::get_cargo_root;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

use std::io::{Error, ErrorKind};

pub fn run_nasm(output_dir: &Path, filepath: impl AsRef<Path>) -> Result<Vec<u8>, Error> {
    let cargo_root = get_cargo_root()?;
    let nasm = cargo_root.join("extras").join("nasm").join("nasm.exe");

    let temp_file = NamedTempFile::new_in(output_dir)?;

    let output = Command::new(nasm)
        .args(["-o", temp_file.path().to_str().unwrap()])
        .arg(filepath.as_ref().as_os_str())
        .output()?;

    if !output.status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!(
                "running nasm on {:?}: {:?}",
                filepath.as_ref(),
                String::from_utf8(output.stderr).unwrap(),
            ),
        ));
    }

    let bytes = std::fs::read(&temp_file)?;
    Ok(bytes)
}
