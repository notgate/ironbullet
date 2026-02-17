fn main() {
    // Force cargo to re-run (and thus re-expand include_dir! macro)
    // whenever any file in gui/build changes.
    println!("cargo:rerun-if-changed=gui/build");
}
