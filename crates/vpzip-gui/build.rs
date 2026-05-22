fn main() {
    #[cfg(windows)]
    {
        // Tag the executable with a versioninfo block, an icon and a manifest declaring
        // Windows 7+ compatibility. The manifest is what makes the binary register cleanly with
        // shell-integration and prevents UAC virtualisation on legacy targets.
        let mut res = winres::WindowsResource::new();
        let icon = std::path::Path::new("assets").join("vpzip.ico");
        if icon.exists() {
            res.set_icon(icon.to_str().unwrap());
        }
        res.set(
            "FileDescription",
            "VPZip \u{2014} Valve Source engine archiver",
        );
        res.set("ProductName", "VPZip");
        res.set("OriginalFilename", "VPZip.exe");
        res.set("InternalName", "VPZip");
        res.set("CompanyName", "VPZip contributors");
        res.set(
            "LegalCopyright",
            "Released under the MIT License. See LICENSE.",
        );
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.set_manifest(
            r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <!-- Windows Vista / 7 / 8 / 8.1 / 10 / 11 -->
      <supportedOS Id="{e2011457-1546-43c5-a5fe-008deee3d3f0}"/>
      <supportedOS Id="{35138b9a-5d96-4fbd-8e2d-a2440225f93a}"/>
      <supportedOS Id="{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}"/>
      <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}"/>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">permonitorv2,permonitor</dpiAwareness>
      <longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware>
    </windowsSettings>
  </application>
</assembly>
            "#,
        );
        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=winres failed: {e}");
        }
    }
}
