use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=burin.ico");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let res_obj = format!("{}/app_res.o", out_dir);

    let status = Command::new("windres")
        .args(["app.rc", "-o", &res_obj])
        .status();

    if let Ok(s) = status {
        if s.success() {
            println!("cargo:rustc-link-arg={}", res_obj);
        }
    }
}
