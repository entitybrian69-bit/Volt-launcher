fn main() {
    #[cfg(windows)]
    {
        let icon = "../../packaging/windows/volt.ico";
        println!("cargo:rerun-if-changed={icon}");
        winresource::WindowsResource::new()
            .set_icon(icon)
            .set("ProductName", "Volt Launcher")
            .compile()
            .expect("embed Volt icon");
    }
}
