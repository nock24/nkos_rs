use std::fs;

fn main() {
    let mut c_asm_build = cc::Build::new();

    c_asm_build
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

    c_asm_build.file("src/boot/boot.S");
    for entry in fs::read_dir("src/c").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) == Some("c") {
            c_asm_build.file(path);
        }
    }

    c_asm_build.compile("c_asm");

    println!("cargo:rerun-if-changed=src/c");
    println!("cargo:rerun-if-changed=src/boot/boot.S");
    println!("cargo:rustc-link-arg=-Tlinker.ld");
}
