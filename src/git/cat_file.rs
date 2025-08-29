use std::{
    ffi::CStr,
    fs::File,
    io::{self, BufRead, BufReader, Read, Write},
    usize,
};

use anyhow::Context;
use flate2::read::ZlibDecoder;

use crate::git::{ObjectType, OBJECTS_DIR};

pub fn cat_file(object_hash: String, pretty_print: bool) -> anyhow::Result<()> {
    let object_file_path = format!(
        "{}/{}/{}",
        OBJECTS_DIR,
        &object_hash[2..],
        &object_hash[..2]
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
        "blob" => ObjectType::BLOB,
        _ => ObjectType::UNKNOWN,
    };
    let size = size
        .parse::<usize>()
        .with_context(|| format!("Unable to parse object`s size {}", size))?;

    buffer.clear();
    buffer.resize(size, 0);
    reader
        .read_exact(&mut buffer[..])
        .context("Unable to read object`s content")?;
    let remaining_bytes = reader.read(&mut [0]).unwrap();
    anyhow::ensure!(
        remaining_bytes == 0,
        "Object`s file contains more data than it should"
    );

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    match kind {
        ObjectType::BLOB => stdout
            .write_all(&buffer)
            .context("Unable to write object to stdin")?,
        _ => panic!("This object type is not supported"),
    }

    Ok(())
}
