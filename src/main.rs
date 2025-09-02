use clap::Parser;

use crate::cli::{CliArguments, Command};

mod cli;
mod commands;

fn main() -> anyhow::Result<()> {
    let args = CliArguments::parse();

    match args.command {
        Command::Init => commands::init::invoke(),
        Command::CatFile {
            pretty_print,
            object_hash,
        } => commands::cat_file::invoke(object_hash, pretty_print),
        Command::HashObject { write, object_path } => {
            commands::hash_object::invoke(object_path, write)
        }
    }
}
