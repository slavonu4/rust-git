use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArguments {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Init,
    CatFile {
        #[arg(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
    HashObject {
        #[arg(short = 'w')]
        write: bool,

        file_path: PathBuf,
    },
    LsTree {
        #[arg(long = "name-only")]
        name_only: bool,

        tree_hash: String,
    },
    WriteTree,
    CommitTree {
        #[arg(short = 'm')]
        message: String,

        #[arg(short = 'p')]
        parent_hash: Option<String>,

        tree_hash: String,
    },
}
