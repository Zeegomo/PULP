use std::path::Path;
use std::process::Command;
// const WRAPPER_LIB_DIR: &str = "wrapper/BUILD/GAP8_V2/GCC_RISCV_PULPOS";
const WRAPPER_LIB_DIR: &str = "wrapper/BUILD/GAP8_V2/GCC_RISCV_FREERTOS";

fn main() {
    // Command::new("make")
    //     .args(&["clean", "all"])
    //     .current_dir("wrapper")
    //     .status()
    //     .unwrap();
    Command::new("bash")
        .args(&["-c", "build.sh"])
        .current_dir(&Path::new(WRAPPER_LIB_DIR))
        .status()
        .unwrap();
    // Command::new("ar")
    //     .args(&["crus", "libwrapper.a", "wrapper.o"])
    //     .current_dir(&Path::new(WRAPPER_LIB_DIR))
    //     .status()
    //     .unwrap();
    let cur_dir = std::env::current_dir()
        .expect("unable to get current dir")
        .join(WRAPPER_LIB_DIR);
    println!("cargo:rustc-link-search={}", cur_dir.display());
    println!("cargo:rustc-link-lib=static=wrapper");
}
