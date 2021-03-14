extern crate cmake;

use cmake::Config;
use std::{env, path::PathBuf, process::Command};

const OPENAL_SOFT_TAG: &'static str = "openal-soft-1.19.1";

const OPENAL_REPO: &'static str = "https://github.com/kcat/openal-soft.git";

fn clone_openalsoft() -> PathBuf {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("openal-soft");
    let status = Command::new("git")
        .arg("clone")
        .args(&["--branch", OPENAL_SOFT_TAG])
        .args(&["--depth", "1"])
        .arg(OPENAL_REPO)
        .arg(&out)
        .status()
        .unwrap();
    if !status.success() {
        let status = Command::new("git")
            .arg("clean")
            .arg("-fdx")
            .current_dir(&out)
            .status()
            .unwrap();
        assert!(status.success(), "Failed to clone openal-soft");
        let status = Command::new("git")
            .arg("checkout")
            .arg(format!("tags/{}", OPENAL_SOFT_TAG))
            .current_dir(&out)
            .status()
            .unwrap();
        assert!(status.success(), "Failed to clone openal-soft");
    }
    out
}

fn build_openalsoft(openal_dir: PathBuf) {
    let dst = Config::new(openal_dir)
        .define("ALSOFT_UTILS", "OFF")
        .define("ALSOFT_EXAMPLES", "OFF")
        .define("ALSOFT_TESTS", "OFF")
        .define("LIBTYPE", "SHARED")
        .no_build_target(true)
        .build();
    println!(
        "cargo:rustc-link-search=native={}/build/Debug",
        dst.display()
    );
    println!("cargo:rustc-link-lib=dylib=common");
    println!("cargo:rustc-link-lib=dylib=OpenAl32");
}

fn main() {
    let repo_path = clone_openalsoft();
    build_openalsoft(repo_path)
}
