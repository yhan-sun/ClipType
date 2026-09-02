# ClipType Windows Package

This directory defines the per-user Windows x86_64 package used by development smoke tests and the public beta release workflow.

## Public release assets

A public release ZIP is produced only by `.github/workflows/windows-release.yml` from the exact `main` commit declared by `release/VERSION`.

The release publishes:

- versioned ZIP package and portable executable;
- `SHA256SUMS.txt`;
- Sigstore keyless signature bundles;
- GitHub artifact attestations;
- dependency/license inventory and build metadata;
- matching release notes and compatibility limitations.

The first beta is not Authenticode publisher-signed. Verify Sigstore and GitHub provenance before installation; Windows reputation warnings may still appear.

## Portable use

Run `cliptype.exe`. The native tray owns product settings and controlled shutdown. Configuration is stored separately at `%LOCALAPPDATA%\ClipType\config.toml`.

## Per-user installation

From an extracted release archive:

```powershell
.\install.ps1 -SourceExe .\cliptype.exe
```

Optional start at login:

```powershell
.\install.ps1 -SourceExe .\cliptype.exe -StartAtLogin
```

The executable is copied to `%LOCALAPPDATA%\Programs\ClipType` by default. The script does not request administrator privileges.

Use `-InstallRoot` to select another current-user-writable directory. Use `-NoLaunch` for automation or when the executable should not start immediately.

## Upgrade

Run the installer again with the new release executable. Existing settings are preserved. The explicit `-StartAtLogin` selection updates both the settings file and product-owned current-user Run value.

## Uninstall

```powershell
.\uninstall.ps1
```

Settings are preserved by default. To remove them too:

```powershell
.\uninstall.ps1 -RemoveSettings
```

The uninstaller removes only ClipType-owned installation state and the current-user startup value that points to the installation being removed.

## Verification boundary

Development archives created by `P2 Windows Package` are unsigned smoke-test artifacts and are not public releases. Only versioned assets attached to the GitHub Release, with matching checksums, Sigstore bundles, and GitHub attestations, are public beta artifacts.

Package smoke covers build, script parsing, isolated install, controlled GUI-subsystem start/stop, startup persistence, uninstall rollback, and distributable privacy-sentinel scanning. It does not assert universal per-application compatibility; see `docs/COMPATIBILITY.md`.
