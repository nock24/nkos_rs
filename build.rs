use std::fs;

fn main() {
    let mut cbuild = cc::Build::new();

    cbuild
        .compiler("aarch64-none-elf-gcc")
        .flags([
            "-O3",
            "-ffreestanding",
            "-nostdlib",
            "-nostartfiles",
            "-mgeneral-regs-only",
            "-w",
        ])
        .include("include");

    cbuild.file("src/boot.S");
    for entry in fs::read_dir("src/c").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) == Some("c") {
            cbuild.file(path);
        }
    }

    cbuild.compile("clib");

    println!("cargo:rerun-if-changed=src/cdrivers");
    println!("cargo:rerun-if-changed=src/boot.S");
    println!("cargo:rustc-link-arg=-Tlinker.ld");
}
