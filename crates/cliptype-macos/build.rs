fn main() {
    println!("cargo:rerun-if-changed=native/cliptype_macos.m");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }

    cc::Build::new()
        .file("native/cliptype_macos.m")
        .flag("-fobjc-arc")
        .flag("-Werror=implicit-function-declaration")
        .compile("cliptype_macos_native");

    for framework in [
        "AppKit",
        "ApplicationServices",
        "Carbon",
        "CoreGraphics",
        "Foundation",
        "ServiceManagement",
    ] {
        println!("cargo:rustc-link-lib=framework={framework}");
    }
}
