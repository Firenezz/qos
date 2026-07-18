use std::path::PathBuf;

fn main() {
    let _arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    // Workspace root: crates/kernel/qos-kernel → ../../..
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace = manifest
        .ancestors()
        .nth(3)
        .expect("qos-kernel should live at crates/kernel/qos-kernel");

    let linker_script = workspace.join("arch/linkers/linker.ld");
    let maps_dir = workspace.join("build").join("maps");
    let map_path = maps_dir.join("qos-kernel.map");

    let _ = std::fs::create_dir_all(&maps_dir);

    // Absolute paths so rust-lld finds the script regardless of cwd.
    println!("cargo:rustc-link-arg=-T{}", linker_script.display());
    println!("cargo:rustc-link-arg=--entry=boot_main");
    println!("cargo:rustc-link-arg=-Map={}", map_path.display());

    println!("cargo:rerun-if-changed={}", linker_script.display());
}
