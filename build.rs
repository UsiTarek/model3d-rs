use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=model3d/m3d.h");
    println!("cargo:rerun-if-changed=m3d.c");
    println!("cargo:rustc-link-search={}", env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-lib=static=m3d");

    cc::Build::new()
        .warnings(false)
        .opt_level(2)
        .define("M3D_EXPORTER", None)
        .file("m3d.c")
        .compile("m3d");

    let bindings = bindgen::Builder::default()
        .header("model3d/m3d.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from("src/");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
