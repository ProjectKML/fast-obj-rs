#[cfg(feature = "generate_bindings")]
use std::env;
use std::{fs, path::Path};

fn main() {
    let mut build = cc::Build::new();

    build.include("src");
    build.file("fast_obj/fast_obj.c");

    let target = env::var("TARGET").unwrap();

    if target.starts_with("wasm32") {
        build.flag("-isystem").flag("include_wasm32");
        let host = env::var("HOST").unwrap();
        if host.contains("windows") {
            build.archiver("llvm-ar");
        }
    }

    build.compile("fast_obj_c");

    generate_bindings("gen/bindings.rs");
}

#[cfg(feature = "generate_bindings")]
fn generate_bindings(output_file: &str) {
    let bindings = bindgen::Builder::default()
        .header("fast_obj/fast_obj.h")
        .rustfmt_bindings(true)
        .derive_debug(true)
        .impl_debug(true)
        .blocklist_type("__darwin_.*")
        .allowlist_function("fast_obj.*")
        .trust_clang_mangling(false)
        .layout_tests(false)
        .generate()
        .expect("Failed to generate bindings");

    fs::create_dir_all("gen").unwrap();
    bindings.write_to_file(Path::new(output_file)).expect("Failed to save bindings to file");
}

#[cfg(not(feature = "generate_bindings"))]
fn generate_bindings(_: &str) {}
