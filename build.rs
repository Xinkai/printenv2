extern crate bindgen;
use std::env;
use std::path::PathBuf;

fn bindgen_generate(header: &str) -> Result<bindgen::Bindings, bindgen::BindgenError> {
    let target = env::var("TARGET");

    let mut result = bindgen::Builder::default()
        .header(header)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    if let Ok(target) = target {
        result = result.clang_arg(format!("--target={target}"));
    }
    result.generate()
}

fn main() {
    println!("cargo::rustc-check-cfg=cfg(unix_apple_sysctl)");
    println!("cargo::rustc-check-cfg=cfg(debugger_helper)");
    println!("cargo::rustc-check-cfg=cfg(remote_env)");
    println!("cargo::rustc-check-cfg=cfg(unix_kvm)");

    let unix = env::var("CARGO_CFG_UNIX");
    let os = env::var("CARGO_CFG_TARGET_OS");

    #[allow(clippy::match_same_arms)]
    match (unix.as_deref(), os.as_deref()) {
        (Ok(_), Ok("linux")) => {
            println!("cargo:rustc-cfg=debugger_helper");
            println!("cargo:rustc-cfg=remote_env");
        }
        (Ok(_), Ok("macos")) => {
            println!("cargo:rerun-if-changed=src/apple-sysctl-wrapper.h");

            if let Ok(sysctl_bindings) = bindgen_generate("src/apple-sysctl-wrapper.h") {
                let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
                sysctl_bindings
                    .write_to_file(out_path.join("apple-sysctl-bindings.rs"))
                    .expect("Couldn't write apple-sysctl-bindings!");
                println!("cargo:rustc-cfg=unix_apple_sysctl");
                println!("cargo:rustc-cfg=remote_env");
            }

            println!("cargo:rustc-cfg=debugger_helper");
        }
        (Ok(_), _) => {
            println!("cargo:rerun-if-changed=src/kvm-wrapper.h");

            if let Ok(kvm_bindings) = bindgen_generate("src/kvm-wrapper.h") {
                println!("cargo:rustc-link-lib=kvm");
                let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
                kvm_bindings
                    .write_to_file(out_path.join("kvm-bindings.rs"))
                    .expect("Couldn't write kvm-bindings!");
                println!("cargo:rustc-cfg=unix_kvm");
                println!("cargo:rustc-cfg=remote_env");
            }

            println!("cargo:rustc-cfg=debugger_helper");
        }
        (Err(_), Ok("windows")) => {
            println!("cargo:rustc-cfg=remote_env");
        }
        _ => (),
    }
}
