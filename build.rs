use std::{env, fs, path::PathBuf};

fn main() {
    if env::var("TARGET").expect("TARGET is set by Cargo") != "thumbv7em-none-eabihf" {
        return;
    }

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    fs::copy("memory.x", out.join("memory.x")).expect("copy memory.x");

    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rustc-link-arg=--nmagic");
    println!("cargo:rustc-link-arg=-Tlink.x");
}
