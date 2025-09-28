use crate::OBJECTS_DIR;
use anyhow::Context;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::{
    ffi::CStr,
    fmt::Display,
    fs::{self, File},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Unknown,
}

pub struct Object<R> {
    pub kind: ObjectType,
    pub expected_size: u64,
    pub reader: R,
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match self {
            Self::Blob => "blob",
            Self::Tree => "tree",
            Self::Commit => "commit",
            _ => "unknown",
        };
        f.write_str(result)
    }
}

pub fn blob_from_file(file_path: &Path) -> anyhow::Result<Object<impl Read>> {
    let file = fs::File::open(file_path).context("Can not read the file")?;
    let file_metadata = file.metadata().context("Can not get file metadata")?;
    Ok(Object {
        kind: crate::objects::ObjectType::Blob,
        expected_size: file_metadata.len(),
        reader: file,
    })
}

pub fn read_object(object_hash: &str) -> anyhow::Result<Object<impl Read>> {
    let object_file_path = format!(
        "{}/{}/{}",
        OBJECTS_DIR,
        &object_hash[..2],
        &object_hash[2..]
    );

    let object_file = File::open(object_file_path).context("Unable to read object file")?;
    let decoder = ZlibDecoder::new(object_file);
    let mut reader = BufReader::new(decoder);
    let mut buffer = Vec::new();

    reader
        .read_until(0, &mut buffer)
        .context("Unable to read object`s header")?;

    let header = CStr::from_bytes_with_nul(&buffer).context("Unable to parse object`s header")?;
    let header = header
        .to_str()
        .context("Object`s header is not a valid UTF-8 string")?;
    let header_parts = header.split_once(' ');
    anyhow::ensure!(
        header_parts.is_some(),
        "Object`s header is malformed {header}"
    );
    let (kind, size) = header_parts.unwrap();
    let kind = match kind {
        "blob" => ObjectType::Blob,
        "tree" => ObjectType::Tree,
        "commit" => ObjectType::Commit,
        _ => ObjectType::Unknown,
    };
    let size = size
        .parse::<u64>()
        .with_context(|| format!("Unable to parse object`s size {}", size))?;

    Ok(Object {
        kind,
        expected_size: size,
        reader,
    })
}

impl<R> Object<R>
where
    R: Read,
{
    pub fn write<W: Write>(self, writer: W) -> anyhow::Result<String> {
        let header = format!("{} {}\0", self.kind, self.expected_size);

        let mut reader = BufReader::new(self.reader);
        let mut file_content = String::new();

        reader
            .read_to_string(&mut file_content)
            .context("Can not read file`s content")?;
        let object_content = format!("{}{}", header, file_content);

        let mut hasher = Sha1::new();
        hasher.update(&object_content);
        let object_hash = hasher.finalize();
        let object_hash = hex::encode(object_hash);

        let mut encoder = ZlibEncoder::new(writer, Compression::default());
        write!(encoder, "{}", object_content).context("Unable to compress object`s hash")?;
        encoder.finish().context("Unable to write the object")?;
        Ok(object_hash)
    }

    pub fn write_to_objects_dir(self) -> anyhow::Result<String> {
        let tmp = "temporary";
        let temp_file = fs::File::create(tmp).context("Unable to create a tmp file")?;
        let object_hash = self
            .write(&temp_file)
            .context("Unable to write to a tmp file")?;

        let object_path = format!(
            "{}/{}/{}",
            OBJECTS_DIR,
            &object_hash[..2],
            &object_hash[2..]
        );
        let object_path = PathBuf::from(object_path);
        fs::create_dir_all(object_path.parent().unwrap())
            .context("Can not create a directory for the object")?;
        fs::rename(tmp, object_path).context("Can not move object file from temp")?;

        Ok(object_hash)
    }
}
