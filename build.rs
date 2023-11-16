use std::process::Command;
fn main() {
    if !cfg!(debug_assertions){
        println!("cargo:rustc-rerun-if-changed=.git/HEAD");
        // note: add error checking yourself.
        let output = Command::new("git").args(&["rev-parse", "HEAD"]).output().unwrap();
        let mut git_hash = String::from_utf8(output.stdout).unwrap();
        git_hash.truncate(6);
        println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    }
}