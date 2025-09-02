use clap::Parser;

use crate::{
    cli::{CliArguments, Command},
    git::{cat_file::cat_file, hash_object::hash_object, init::init_git_directory},
};

mod cli;
mod git;

fn main() -> anyhow::Result<()> {
    let args = CliArguments::parse();

    match args.command {
        Command::Init => init_git_directory(),
        Command::CatFile {
            pretty_print,
            object_hash,
        } => cat_file(object_hash, pretty_print),
        Command::HashObject { write, object_path } => hash_object(object_path, write),
    }
}
