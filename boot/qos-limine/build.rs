
fn main() {
    // Tell cargo to pass the linker script to the linker..
    println!("cargo:rustc-link-arg=-Tlinker.ld");
    // ..and to re-run if it changes.
    println!("cargo:rerun-if-changed=linker.ld");

    // Tell cargo to make rustc output a link map.
    println!("cargo:rustc-link-arg=-Map=qos-limine.map");
}