use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=src/net.c");
    println!("cargo:rerun-if-env-changed=CATLIONS_DROP_TCP");
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest.join("../../..");
    let sdk = env::var("MICROKIT_SDK").expect("MICROKIT_SDK is required");
    let board = env::var("MICROKIT_BOARD").unwrap_or_else(|_| "qemu_virt_aarch64".into());
    let config = env::var("MICROKIT_CONFIG").unwrap_or_else(|_| "debug".into());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let obj = out.join("sddf_net.o");
    let memcpy = out.join("memcpy.o");
    let memset = out.join("memset.o");
    let lib = out.join("libcatlions_sddf.a");
    let drop_tcp = env::var("CATLIONS_DROP_TCP").unwrap_or_else(|_| "0".into());
    let status = Command::new("clang")
        .args([
            "--target=aarch64-none-elf",
            "-std=gnu11",
            "-O2",
            "-DNDEBUG",
            "-ffreestanding",
            "-fno-builtin-memcmp",
            "-fno-builtin-strlen",
            "-c",
        ])
        .arg(manifest.join("src/net.c"))
        .arg(format!("-DCATLIONS_DROP_TCP={drop_tcp}"))
        .arg(format!("-I{sdk}/board/{board}/{config}/include"))
        .arg(format!("-I{}/dep/sddf/include", root.display()))
        .arg(format!("-I{}/dep/sddf/include/microkit", root.display()))
        .arg(format!(
            "-I{}/dep/sddf/include/sddf/util/custom_libc",
            root.display()
        ))
        .arg("-o")
        .arg(&obj)
        .status()
        .unwrap();
    assert!(status.success());
    for (name, output) in [("memcpy", &memcpy), ("memset", &memset)] {
        assert!(
            Command::new("clang")
                .arg("--target=aarch64-none-elf")
                .arg("-c")
                .arg(root.join(format!("dep/sddf/util/custom_libc/aarch64/{name}.S")))
                .arg("-o")
                .arg(output)
                .status()
                .unwrap()
                .success()
        );
    }
    assert!(
        Command::new("llvm-ar")
            .arg("crs")
            .arg(&lib)
            .args([&obj, &memcpy, &memset])
            .status()
            .unwrap()
            .success()
    );
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=catlions_sddf");
}
