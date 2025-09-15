use std::io::{self, BufRead, BufReader, Read, Write};

use anyhow::{ensure, Context};

use crate::objects::{self, read_object, ObjectType};

pub fn invoke(tree_hash: String, name_only: bool) -> anyhow::Result<()> {
    let object = read_object(&tree_hash).context("Can not read the given object")?;

    ensure!(
        object.kind == ObjectType::Tree,
        "The given object is not a tree"
    );

    let mut reader = BufReader::new(object.reader);
    let mut name_and_mode_buf = Vec::new();
    let mut hash_buf = [0; 20];
    let mut stdout = io::stdout().lock();
    loop {
        name_and_mode_buf.clear();
        let bytes_read = reader
            .read_until(0, &mut name_and_mode_buf)
            .context("Unable to read name and mode of a tree entry")?;
        if bytes_read == 0 {
            break;
        }

        reader
            .read_exact(&mut hash_buf)
            .context("Unable to read hash of a tree entry")?;

        let output_line = if name_only {
            get_name(&name_and_mode_buf)
        } else {
            get_full_output(&name_and_mode_buf, &hash_buf)
        }
        .context("Tree entry is malformed")?;

        stdout
            .write_all(&output_line)
            .context("Unable to write to stdout")?;

        writeln!(stdout).context("Unable to write to stdout")?;
    }

    Ok(())
}

fn get_name(buf: &Vec<u8>) -> anyhow::Result<Vec<u8>> {
    get_mode_and_name(buf).map(|mn| mn.1)
}

fn get_full_output(mode_and_name: &Vec<u8>, hash: &[u8]) -> anyhow::Result<Vec<u8>> {
    let (mode, mut name) = get_mode_and_name(mode_and_name)?;

    let mut mode = get_formatted_mode(mode);
    let decoded_hash = hex::encode(hash);
    let object = objects::read_object(&decoded_hash)
        .context("Unable to get data for an object in the tree")?;
    let kind = object.kind.to_string();

    let mut result: Vec<u8> = Vec::new();

    result.append(&mut mode);
    result.push(b' ');
    result.append(&mut kind.as_bytes().to_vec());
    result.push(b' ');
    result.append(&mut decoded_hash.as_bytes().to_vec());
    result.push(b'\x09');
    result.append(&mut name);

    Ok(result)
}

fn get_mode_and_name(buf: &Vec<u8>) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
    let mut mode_and_name = buf.splitn(2, |&b| b == b' ');
    let mode = mode_and_name
        .next()
        .map(|a| a.to_vec())
        .ok_or_else(|| anyhow::anyhow!("Tree entry is malformed"))?;

    let name = mode_and_name
        .next()
        .map(|a| a.to_vec())
        .ok_or_else(|| anyhow::anyhow!("Tree entry is malformed"))?;

    Ok((mode, name))
}

fn get_formatted_mode(mode: Vec<u8>) -> Vec<u8> {
    let padding_size = 6 - mode.len();

    if padding_size <= 0 {
        return mode;
    }

    let mut result = Vec::new();

    for _ in 0..padding_size {
        result.push(b'0');
    }

    for byte in mode {
        result.push(byte);
    }

    result
}
