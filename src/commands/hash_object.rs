use std::{
    fs,
    io::{self},
    path::PathBuf,
};

use anyhow::Context;

use crate::{objects::blob_from_file, OBJECTS_DIR};

pub fn invoke(file_path: PathBuf, write: bool) -> anyhow::Result<()> {
    let object = blob_from_file(file_path).context("Unable to read the file")?;

    let mut temp_file = fs::File::create("temporary").context("Unable to create a tmp file")?;
    let object_hash = object
        .write(&temp_file)
        .context("Unable to write to a tmp file")?;
    if write {
        let object_path = format!(
            "{}/{}/{}",
            OBJECTS_DIR,
            &object_hash[..2],
            &object_hash[2..]
        );
        let object_path = PathBuf::from(object_path);
        fs::create_dir_all(object_path.parent().unwrap())
            .context("Can not create a directory for the object")?;
        let mut object_file =
            fs::File::create(object_path).context("Can not create object`s file")?;
        io::copy(&mut temp_file, &mut object_file)
            .context("Unable to write to the object`s file")?;
    }

    println!("{}", object_hash);
    Ok(())
}
