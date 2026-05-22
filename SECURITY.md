# Security Policy

## Supported versions

Only the latest released version of VPZip receives security fixes.

## Reporting

Please report security vulnerabilities **privately** via GitHub's
[private security advisory](https://github.com/hohlov2006362018-arch/SourceVault/security/advisories/new)
flow rather than opening a public issue.

The kinds of issues we treat as security-critical:

- Memory-safety bugs in archive parsers (panic / OOB read / OOB write on a crafted archive).
- Path traversal during extraction (writing outside the destination directory).
- Code execution triggered by opening or extracting a hostile archive.

## Out of scope

- Antivirus false positives. See the *VirusTotal & code signing* section of the README — please
  open a regular issue with the engine and signature name so we can submit a clean-file report
  to the vendor.
- Issues in upstream crates we depend on (`eframe`, `image`, `flate2`, …). Please file those with
  the upstream project; we will track them via dependency bumps.
