fn main() {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    // Tell cargo to pass the linker script to the linker..
    println!("cargo:rustc-link-arg=-Tlinker.ld");
    // ..and to re-run if it changes.
    println!("cargo:rerun-if-changed=linker.ld");
    println!("cargo:rustc-link-arg=-Map=output.map");
}