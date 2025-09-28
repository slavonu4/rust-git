use std::path::Path;

use anyhow::Context;

use crate::objects;

pub fn invoke(message: String) -> anyhow::Result<()> {
    let parent_hash =
        objects::get_head_hash().context("Unable to get parent hash for the commit")?;

    let Some(tree_hash) =
        objects::write_tree(Path::new(".")).context("Unable to write the tree")?
    else {
        eprintln!("Not committing an empty tree");
        return Ok(());
    };

    let commit_hash = objects::write_commit(tree_hash, Some(parent_hash), message)
        .context("Can not write the commit")?;

    objects::update_head_ref(&commit_hash)?;

    println!("HEAD is now at {commit_hash}");

    Ok(())
}
