use crate::OBJECTS_DIR;
use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::{
    ffi::CStr,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader, Read},
};

#[derive(Debug, PartialEq, Eq)]
pub enum ObjectType {
    Blob,
    Tree,
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
            _ => "unknown",
        };
        f.write_str(result)
    }
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
