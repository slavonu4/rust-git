use std::{
    cmp::{self, Ordering},
    ffi::OsString,
    fs::{self, DirEntry, Metadata},
    io::Cursor,
    os::unix::fs::PermissionsExt,
    path::Path,
};

use anyhow::{Context, Ok};

use crate::objects::{self, Object};

pub fn invoke() -> anyhow::Result<()> {
    let Some(hash) = write_tree(Path::new(".")).context("Unable to write a tree")? else {
        return Ok(());
    };

    println!("{}", hash);

    Ok(())
}

fn write_tree(path: &Path) -> anyhow::Result<Option<String>> {
    let dir =
        fs::read_dir(path).with_context(|| format!("Unable to read dir {}", path.display()))?;

    let mut entries = Vec::new();

    for entry in dir {
        let entry = entry.with_context(|| format!("Bad dir entry in {}", path.display()))?;
        let file_name = entry.file_name();
        let metadata = entry.metadata().with_context(|| {
            format!(
                "Can not get metadata of the entry {} in {}",
                file_name.display(),
                path.display()
            )
        })?;
        entries.push(DirEntryInfo {
            entry,
            file_name,
            metadata,
        });
    }

    entries.sort_unstable_by(compare_dir_entries);

    let mut tree_object = Vec::new();
    for entry in entries {
        let file_name = entry.file_name;

        if file_name == ".git" || file_name == crate::GIT_DIR {
            continue;
        }

        let dir_entry = entry.entry;
        let entry_path = dir_entry.path();
        let metadata = entry.metadata;
        let mode = get_mode(&metadata);

        let hash = if metadata.is_dir() {
            let Some(hash) = write_tree(&entry_path)? else {
                continue;
            };

            hash
        } else {
            let object = objects::blob_from_file(&entry_path).with_context(|| {
                format!("Bad blob {} in {}", entry_path.display(), path.display())
            })?;

            object
                .write_to_objects_dir()
                .with_context(|| format!("Unable to write object {}", entry_path.display()))?
        };

        tree_object.extend(mode.as_bytes());
        tree_object.push(b' ');
        tree_object.extend(file_name.as_encoded_bytes());
        tree_object.push(0);
        tree_object.extend(hex::decode(hash)?);
    }

    if tree_object.is_empty() {
        return Ok(None);
    }

    let hash = Object {
        kind: objects::ObjectType::Tree,
        expected_size: tree_object.len() as u64,
        reader: Cursor::new(tree_object),
    }
    .write_to_objects_dir()
    .with_context(|| format!("Unable to write tree objet {}", path.display()))?;

    Ok(Some(hash))
}

struct DirEntryInfo {
    entry: DirEntry,
    file_name: OsString,
    metadata: Metadata,
}

fn get_mode(metadata: &Metadata) -> &str {
    if metadata.is_dir() {
        "40000"
    } else if metadata.is_symlink() {
        "120000"
    } else if (metadata.permissions().mode() & 0o111) != 0 {
        "100755"
    } else {
        "100644"
    }
}

fn compare_dir_entries(a: &DirEntryInfo, b: &DirEntryInfo) -> Ordering {
    let a_file_name = a.file_name.as_encoded_bytes();
    let b_file_name = b.file_name.as_encoded_bytes();
    let common_length = cmp::min(a_file_name.len(), b_file_name.len());

    match a_file_name[..common_length].cmp(&b_file_name[..common_length]) {
        Ordering::Equal => {}
        o => return o,
    }

    let c1 = if let Some(c) = a_file_name.get(common_length).copied() {
        Some(c)
    } else if a.metadata.is_dir() {
        Some(b'/')
    } else {
        None
    };

    let c2 = if let Some(c) = b_file_name.get(common_length).copied() {
        Some(c)
    } else if b.metadata.is_dir() {
        Some(b'/')
    } else {
        None
    };

    c1.cmp(&c2)
}
