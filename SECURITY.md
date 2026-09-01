# Security Policy

ClipType interacts with sensitive OS surfaces: the clipboard, global hotkeys, focused applications, synthetic input, and on some platforms accessibility or virtual input permissions.

## Reporting a vulnerability

Until a dedicated private reporting channel is configured, do not publish exploit details or sensitive user data in a public issue. Contact the repository maintainer privately through an available GitHub channel and provide a minimal reproduction, affected platform/version, impact, and suggested mitigation if known.

## Security boundaries

ClipType must not attempt to bypass platform security controls. Examples include Windows UIPI, macOS Accessibility consent, Wayland compositor policy, and Linux device permissions.

## Sensitive data

Clipboard text and injected text are sensitive by default. They must not be written to logs, telemetry, crash annotations, analytics, clipboard history, or persistent debug artifacts.

See `docs/SECURITY_PRIVACY.md` for the normative threat model and engineering requirements.