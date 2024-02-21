use std::fs::create_dir;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = "src/clib/";

    match create_dir(out_dir) {
        Ok(_) => println!("Creating C library directory at {out_dir}..."),
        Err(_) => println!("C library directory already exists, skipping..."),
    }

    Command::new("gcc")
        .args(&["src/termc.c", "-c", "-fPIC", "-o"])
        .arg(&format!("{}termc.o", out_dir))
        .status()
        .unwrap();

    // Calling the archive libtermc.a in linux is necessary
    // due to a quirk when compiling on linux

    #[cfg(target_os = "linux")]
    Command::new("ar")
        .args(&["crus", "libtermc.a", "termc.o"])
        .current_dir(&Path::new(out_dir))
        .status()
        .unwrap();

    #[cfg(target_os = "windows")]
    Command::new("ar")
        .args(&["rcs", "termc.lib", "termc.o"])
        .current_dir(&Path::new(out_dir))
        .status()
        .unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=termc");
    println!("cargo:rerun-if-changed=src/termc.c");
}
