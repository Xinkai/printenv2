extern crate bindgen;
use std::env;
use std::path::PathBuf;

fn main() {
    let unix = env::var("CARGO_CFG_UNIX");
    let os = env::var("CARGO_CFG_TARGET_OS");

    #[allow(clippy::match_same_arms)]
    match (unix.as_deref(), os.as_deref()) {
        (Ok(_), Ok("linux")) => (),
        (Ok(_), Ok("macos")) => (),
        (Ok(_), _) => {
            println!("cargo:rerun-if-changed=src/kvm-wrapper.h");

            if let Ok(kvm_bindings) = bindgen::Builder::default()
                .header("src/kvm-wrapper.h")
                .parse_callbacks(Box::new(bindgen::CargoCallbacks))
                .generate()
            {
                println!("cargo:rustc-link-lib=kvm");
                let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
                kvm_bindings
                    .write_to_file(out_path.join("kvm-bindings.rs"))
                    .expect("Couldn't write kvm-bindings!");
                println!("cargo:rustc-cfg=unix_kvm");
            }
        }
        _ => (),
    }
}
