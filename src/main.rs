use clap::Parser;

use crate::cli::{CliArguments, Command};

mod cli;
mod commands;
mod objects;

pub const GIT_DIR: &str = ".rgit";
pub const OBJECTS_DIR: &str = ".rgit/objects";
pub const REFS_DIR: &str = ".rgit/refs";
pub const HEAD: &str = ".rgit/HEAD";

fn main() -> anyhow::Result<()> {
    let args = CliArguments::parse();

    match args.command {
        Command::Init => commands::init::invoke(),
        Command::CatFile {
            pretty_print,
            object_hash,
        } => commands::cat_file::invoke(object_hash, pretty_print),
        Command::HashObject { write, file_path } => commands::hash_object::invoke(file_path, write),
        Command::LsTree {
            name_only,
            tree_hash,
        } => commands::ls_tree::invoke(tree_hash, name_only),
        Command::WriteTree => commands::write_tree::invoke(),
        Command::CommitTree {
            message,
            parent_hash,
            tree_hash,
        } => commands::commit_tree::invoke(tree_hash, parent_hash, message),
    }
}
