# ClipType macOS packaging

`package.sh` assembles an ad-hoc-signed Universal 2 candidate for CI. It does not claim Developer ID identity or notarization. Public distribution must run the protected signing/notarization job in `macos-release.yml`.

The bundle is a menu-bar application (`LSUIElement`) with a Slint settings window. Synthetic input remains disabled until the user grants Accessibility permission in System Settings.
