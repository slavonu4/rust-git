use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::Context;
use flate2::{write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};

use crate::OBJECTS_DIR;

pub fn invoke(object_path: PathBuf, write: bool) -> anyhow::Result<()> {
    let mut file = fs::File::open(&object_path).context("Can not read the file")?;
    let file_metadata = file.metadata().context("Can not get file metadata")?;
    let file_size = file_metadata.len();
    let header = format!("blob {}\0", file_size);
    // TODO: use temp file and a custom writer to avoid reading an entire file in memory
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)
        .context("Can not read file`s content")?;
    let object_content = format!("{}{}", header, file_content);

    let mut hasher = Sha1::new();
    hasher.update(&object_content);
    let object_hash = hasher.finalize();
    let object_hash = hex::encode(object_hash);
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
        let object_file = fs::File::create(object_path).context("Can not create object`s file")?;

        let mut encoder = ZlibEncoder::new(object_file, Compression::default());
        write!(encoder, "{}", object_content).context("Unable to compress object`s hash")?;
        encoder.finish().context("Unable to write the object")?;
    }
    println!("{}", object_hash);
    Ok(())
}
