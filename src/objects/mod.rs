use crate::{objects, OBJECTS_DIR};
use anyhow::Context;
use chrono::Local;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::cmp::{self, Ordering};
use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs::DirEntry;
use std::os::unix::fs::PermissionsExt;
use std::{
    ffi::CStr,
    fmt::Display,
    fs::{self, File, Metadata},
    io::{BufRead, BufReader, Cursor, Read, Write},
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

pub fn write_tree(path: &Path) -> anyhow::Result<Option<String>> {
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

pub fn commit_tree(
    tree_hash: String,
    parent_hash: Option<String>,
    message: String,
) -> anyhow::Result<String> {
    let mut commit_content = String::default();
    let now = Local::now();
    let commit_time = now.timestamp();
    let timezone_offset = now.format("%z").to_string();

    writeln!(commit_content, "tree {tree_hash}")?;
    if let Some(parent_hash) = parent_hash {
        writeln!(commit_content, "parent {parent_hash}")?;
    }
    writeln!(
        commit_content,
        "author Viacheslav Bobrenok <test@test.com> {commit_time} {timezone_offset}"
    )?;
    writeln!(
        commit_content,
        "committer Viacheslav Bobrenok <test@test.com> {commit_time} {timezone_offset}"
    )?;
    writeln!(commit_content)?;
    writeln!(commit_content, "{message}")?;

    let commit_object = Object {
        kind: ObjectType::Commit,
        expected_size: commit_content.len() as u64,
        reader: Cursor::new(commit_content),
    };

    let hash = commit_object
        .write_to_objects_dir()
        .context("Unable to write commit file")?;
    Ok(hash)
}
