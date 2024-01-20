use std::process::Command;
use chrono::prelude::*;

fn main() {
    if !cfg!(debug_assertions){
        println!("cargo:rustc-rerun-if-changed=.git/HEAD");

        let output = Command::new("git").args(&["rev-parse", "HEAD"]).output().unwrap();
        let mut git_hash = String::from_utf8(output.stdout).unwrap();
        git_hash.truncate(6);
        let dt: DateTime<Utc> = Utc::now();  
        println!("cargo:rustc-env=GIT_HASH={}", git_hash);
        println!("cargo:rustc-env=BUILD_VERSION={}", env!("CARGO_PKG_VERSION"));
        println!("cargo:rustc-env=BUILD_TIME={}", dt.to_rfc2822());

    }
}