fn main() {
    // Link Windows libraries required by libgit2
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=advapi32");
    }
}
