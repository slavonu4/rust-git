use std::{
    fs,
    io::{self},
    path::PathBuf,
};

use anyhow::Context;

use crate::{objects::Object, OBJECTS_DIR};

pub fn invoke(object_path: PathBuf, write: bool) -> anyhow::Result<()> {
    let file = fs::File::open(&object_path).context("Can not read the file")?;
    let file_metadata = file.metadata().context("Can not get file metadata")?;
    let object = Object {
        kind: crate::objects::ObjectType::Blob,
        expected_size: file_metadata.len(),
        reader: file,
    };

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
