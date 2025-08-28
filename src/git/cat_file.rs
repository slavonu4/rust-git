use std::{
    ffi::CStr,
    fs::File,
    io::{self, BufRead, BufReader, Read, Write},
    usize,
};

use flate2::read::ZlibDecoder;

use crate::git::{ObjectType, OBJECTS_DIR};

pub fn cat_file(object_hash: String, pretty_print: bool) {
    let object_file_path = format!(
        "{}/{}/{}",
        OBJECTS_DIR,
        &object_hash[2..],
        &object_hash[..2]
    );
    let object_file = File::open(object_file_path).unwrap();
    let decoder = ZlibDecoder::new(object_file);
    let mut reader = BufReader::new(decoder);
    let mut buffer = Vec::new();

    reader.read_until(0, &mut buffer).unwrap();

    let header = CStr::from_bytes_with_nul(&buffer).unwrap();
    let header = header.to_str().unwrap();
    let (kind, size) = header.split_once(' ').unwrap();
    let kind = match kind {
        "blob" => ObjectType::BLOB,
        _ => ObjectType::UNKNOWN,
    };
    let size = size.parse::<usize>().unwrap();

    buffer.clear();
    buffer.resize(size, 0);
    reader.read_exact(&mut buffer[..]).unwrap();
    let remaining_bytes = reader.read(&mut [0]).unwrap();
    assert_eq!(remaining_bytes, 0, "file contains more data than it should");

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    match kind {
        ObjectType::BLOB => stdout.write_all(&buffer).unwrap(),
        _ => panic!("This object type is not supported"),
    }
}
