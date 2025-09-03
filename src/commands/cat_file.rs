use anyhow::Context;
use std::io::{self, Read};

use crate::objects::ObjectType;

pub fn invoke(object_hash: String, pretty_print: bool) -> anyhow::Result<()> {
    anyhow::ensure!(pretty_print, "Pretty print flag is required for now");
    let object = crate::objects::read_object(object_hash).context("Unable to read the object")?;
    anyhow::ensure!(
        object.kind != ObjectType::Unknown,
        "This object type is not supported"
    );

    let mut reader = object.reader.take(object.expected_size);
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    io::copy(&mut reader, &mut stdout).context("Unable to write object to stdout")?;

    Ok(())
}
