use std::fs;

const GIT_DIR: &str = ".rgit";

pub fn init_git_directory() {
    fs::create_dir(GIT_DIR).unwrap();
    fs::create_dir(format!("{}/{}", GIT_DIR, "objects")).unwrap();
    fs::create_dir(format!("{}/{}", GIT_DIR, "refs")).unwrap();
    fs::write(format!("{}/{}", GIT_DIR, "HEAD"), "ref: refs/heads/main\n").unwrap();
    println!("Initialized git directory");
}
