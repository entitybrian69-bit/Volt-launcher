fn main() {
    #[cfg(windows)]
    {
        let icon = format!(
            "{}/../../packaging/windows/volt.ico",
            env!("CARGO_MANIFEST_DIR")
        );
        println!("cargo:rerun-if-changed={icon}");
        winresource::WindowsResource::new()
            .set_icon(&icon)
            .set("ProductName", "Volt Launcher")
            .compile()
            .expect("embed Volt icon");
    }
}
