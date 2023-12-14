use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = "src/clib/";

    Command::new("gcc")
        .args(&["src/termc.c", "-c", "-fPIC", "-o"])
        .arg(&format!("{}termc.o", out_dir))
        .status()
        .unwrap();

    Command::new("ar")
        .args(&["crus", "libtermc.a", "termc.o"])
        .current_dir(&Path::new(out_dir))
        .status()
        .unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=termc");
    println!("cargo:rerun-if-changed=src/termc.c");
}
