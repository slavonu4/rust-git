use clap::Parser;

use crate::{
    cli::{CliArguments, Command},
    git::init::init_git_directory,
};

mod cli;
mod git;

fn main() {
    let args = CliArguments::parse();

    match args.command {
        Command::Init => init_git_directory(),
    }
}
