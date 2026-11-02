// The program icon and version information, when building for Windows on Windows.
fn main() {
    println!("cargo:rerun-if-changed=resources/builder.rc");
    println!("cargo:rerun-if-changed=resources/u3drinth.ico");
    let windows =
        std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows");
    if windows && cfg!(windows) {
        embed_resource::compile("resources/builder.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("resources/builder.rc");
    }
}
