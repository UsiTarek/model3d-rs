use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=m3d_wrapper.h");
    println!("cargo:rustc-link-search={}", env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-lib=static=m3d");

    cc::Build::new()
        .define("M3D_EXPORTER", None)
        .compiler("clang")
        .file("m3d.c")
        .compile("m3d");

    let bindings = bindgen::Builder::default()
        .header("m3d_wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from("src/m3d/");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
