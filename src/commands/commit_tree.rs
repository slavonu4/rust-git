use std::{fmt::Write, io::Cursor};

use anyhow::Context;
use chrono::Local;

use crate::objects::{Object, ObjectType};

pub fn invoke(
    tree_hash: String,
    parent_hash: Option<String>,
    message: String,
) -> anyhow::Result<()> {
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

    println!("{hash}");

    Ok(())
}
