use clap::Parser;

use crate::{
    cli::{CliArguments, Command},
    git::{cat_file::cat_file, init::init_git_directory},
};

mod cli;
mod git;

fn main() {
    let args = CliArguments::parse();

    match args.command {
        Command::Init => init_git_directory(),
        Command::CatFile {
            pretty_print,
            object_hash,
        } => cat_file(object_hash, pretty_print),
    }
}
