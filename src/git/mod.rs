pub mod cat_file;
pub mod init;

pub const GIT_DIR: &str = ".rgit";
pub const OBJECTS_DIR: &str = ".rgit/objects";

pub enum ObjectType {
    BLOB,
    UNKNOWN,
}
