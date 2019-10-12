use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target = env::var("TARGET").unwrap();

    println!("cargo:rustc-cfg=target=\"{}\"", target);

    if target.starts_with("thumb") {
        println!("cargo:rustc-cfg=thumb")
    } else if target.starts_with("arm") {
        println!("cargo:rustc-cfg=arm")
    }
}
