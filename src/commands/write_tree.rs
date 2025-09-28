use std::path::Path;

use anyhow::{Context, Ok};

use crate::objects::{self};

pub fn invoke() -> anyhow::Result<()> {
    let Some(hash) = objects::write_tree(Path::new(".")).context("Unable to write a tree")? else {
        return Ok(());
    };

    println!("{}", hash);

    Ok(())
}
