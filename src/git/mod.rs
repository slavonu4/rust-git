pub mod cat_file;
pub mod init;

pub const GIT_DIR: &str = ".rgit";
pub const OBJECTS_DIR: &str = ".rgit/objects";
pub const REFS_DIR: &str = ".rgit/refs";
pub const HEAD: &str = ".rgit/HEAD";

#[derive(PartialEq, Eq)]
pub enum ObjectType {
    Blob,
    Unknown,
}
