use anyhow::Context;

use crate::commands::{GIT_DIR, HEAD, OBJECTS_DIR, REFS_DIR};
use std::fs;

pub fn invoke() -> anyhow::Result<()> {
    fs::create_dir(GIT_DIR).with_context(|| format!("Unable to create {} directory", GIT_DIR))?;
    fs::create_dir(OBJECTS_DIR)
        .with_context(|| format!("Unable to create {} directory", OBJECTS_DIR))?;
    fs::create_dir(REFS_DIR).with_context(|| format!("Unable to create {} directory", REFS_DIR))?;
    fs::write(HEAD, "ref: refs/heads/main\n")
        .with_context(|| format!("Unable to write to {}", HEAD))?;
    println!("Initialized git directory");
    Ok(())
}
