use anyhow::Context;

use crate::objects;

pub fn invoke(
    tree_hash: String,
    parent_hash: Option<String>,
    message: String,
) -> anyhow::Result<()> {
    let hash = objects::commit_tree(tree_hash, parent_hash, message)
        .context("Unable to commit the tree")?;
    println!("{hash}");

    Ok(())
}
