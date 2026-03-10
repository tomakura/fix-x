fn main() {
    if cfg!(target_os = "windows") {
        println!("cargo:rerun-if-changed=app.manifest");
        let manifest = std::env::current_dir().unwrap().join("app.manifest");
        println!("cargo:rustc-link-arg-bins=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-bins=/MANIFESTINPUT:{}",
            manifest.display()
        );
    }
}
