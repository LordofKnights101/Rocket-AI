//! Build script: embeds the rocket icon into the Windows executable so
//! Explorer, the taskbar and shortcuts all show ROCKET's own icon.

fn main() {
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        winresource::WindowsResource::new()
            .set_icon("assets/rocket.ico")
            .compile()
            .expect("failed to embed assets/rocket.ico into the executable");
    }
}
