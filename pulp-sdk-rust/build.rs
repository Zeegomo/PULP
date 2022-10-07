use std::path::Path;
use std::process::Command;
const WRAPPER_LIB_DIR: &str = "wrapper/BUILD/PULP/GCC_RISCV/wrapper";

fn main() {
    Command::new("make")
        .args(&["clean", "all"])
        .current_dir("wrapper")
        .status()
        .unwrap();
    Command::new("ar")
        .args(&["crus", "libwrapper.a", "wrapper.o"])
        .current_dir(&Path::new(WRAPPER_LIB_DIR))
        .status()
        .unwrap();
    let cur_dir = std::env::current_dir()
        .expect("unable to get current dir")
        .join(WRAPPER_LIB_DIR);
    println!("cargo:rustc-link-search={}", cur_dir.display());
    println!("cargo:rustc-link-lib=static=wrapper");
}
