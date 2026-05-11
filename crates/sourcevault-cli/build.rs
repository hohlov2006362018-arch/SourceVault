fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        if let Ok(repo_root) = std::env::var("CARGO_MANIFEST_DIR") {
            let icon = std::path::Path::new(&repo_root)
                .join("..")
                .join("sourcevault-gui")
                .join("assets")
                .join("sourcevault.ico");
            if icon.exists() {
                res.set_icon(icon.to_str().unwrap());
            }
        }
        res.set("FileDescription", "SourceVault CLI");
        res.set("ProductName", "SourceVault");
        res.set("OriginalFilename", "sourcevault.exe");
        res.set("InternalName", "sourcevault");
        res.set("CompanyName", "SourceVault contributors");
        res.set(
            "LegalCopyright",
            "Released under the MIT License. See LICENSE.",
        );
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=winres failed: {e}");
        }
    }
}
