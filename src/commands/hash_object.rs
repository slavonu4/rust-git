use std::{
    io::{self},
    path::PathBuf,
};

use anyhow::Context;

use crate::objects::blob_from_file;

pub fn invoke(file_path: PathBuf, write: bool) -> anyhow::Result<()> {
    let object = blob_from_file(&file_path).context("Unable to read the file")?;

    let object_hash = if write {
        object.write_to_objects_dir()
    } else {
        object.write(io::sink())
    }
    .context("Unable to get object hash")?;

    println!("{}", object_hash);
    Ok(())
}
