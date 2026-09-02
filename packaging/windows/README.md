# ClipType Windows Development Package

This directory defines the reproducible, unsigned P2 development package. It is not a public release and contains no signing material.

## Portable use

Run `cliptype.exe`. The native tray owns product settings and controlled shutdown. Configuration is stored separately at `%LOCALAPPDATA%\ClipType\config.toml`.

## Per-user installation

```powershell
.\install.ps1 -SourceExe .\cliptype.exe
```

Optional start at login:

```powershell
.\install.ps1 -SourceExe .\cliptype.exe -StartAtLogin
```

The executable is copied to `%LOCALAPPDATA%\Programs\ClipType` by default. No administrator privilege is requested.

## Uninstall

```powershell
.\uninstall.ps1
```

Settings are preserved by default. To remove them too:

```powershell
.\uninstall.ps1 -RemoveSettings
```

The uninstaller removes the HKCU Run entry only when it points to the installation being removed.

## Evidence boundary

The package is suitable only for automated product-gate and explicitly recorded development testing. It does not constitute signing, release promotion, public beta authorization, or broad application compatibility evidence.
