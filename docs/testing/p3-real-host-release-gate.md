# P3 real-host and release gate

This gate closes the gap between a green hosted build and a releasable desktop product. It covers behavior that cannot be established by cross-compilation alone: operating-system permissions, focus changes, global-shortcut conflicts, login items, real application lifecycle, Developer ID signing, notarization, stapling, and Gatekeeper.

A green hosted workflow is necessary but not sufficient. P3 remains in Draft until the exact candidate commit has complete evidence for Windows x86_64, macOS arm64, and macOS x86_64. A public macOS release additionally requires a separate `release-macos` manifest produced after signing and notarization.

## Safety and privacy boundary

Evidence manifests are content-blind. They may contain a GitHub Actions run URL, a repository-relative evidence path, a SHA-256 digest, an opaque local run identifier, or a short command-result reference. They must not contain:

- clipboard contents or a typed test sample;
- screenshots embedded as base64 or binary data;
- usernames, hostnames, IP addresses, device serial numbers, or filesystem paths outside the repository;
- Apple credentials, signing certificates, passwords, tokens, environment dumps, or notarization log bodies;
- production documents or customer data.

The tool records only OS family, OS release, architecture, Python version, the exact commit, test status, and content-blind evidence references. Reports show the number of evidence references, not their values.

## Required matrix

| Manifest platform | Required host | Purpose |
|---|---|---|
| `windows-x86_64` | Clean supported Windows x64 user session | Package lifecycle, live shortcut replacement and rollback, Unicode delivery, focus/integrity boundaries, startup, privacy |
| `macos-arm64` | Apple Silicon macOS session | Native arm64 launch, Accessibility grant/revoke, Secure Event Input, AppKit menu bar, focus evidence, startup |
| `macos-x86_64` | Intel Mac, or the explicitly approved x86_64/Rosetta path | Native x86_64 slice and the same user-visible contract |
| `release-macos` | Controlled release host with protected Apple credentials | Bundle metadata, Universal 2, Hardened Runtime, strict signing verification, notarization, stapling, Gatekeeper, artifact hashes |

Each manifest must represent the same 40-character commit SHA. `verify-set` rejects duplicate platforms, duplicate run identifiers, missing hosts, wrong commits, incomplete required checks, changed catalog requirements, malformed evidence, and manifest tampering.

## 1. Prepare the exact candidate

Use an isolated clone or worktree and start from a clean user account where possible.

```bash
git fetch origin
git checkout --detach <candidate-sha>
test "$(git rev-parse HEAD)" = "<candidate-sha>"
```

Run the permanent hosted checks first. Do not begin real-host sign-off against a commit that is still changing.

## 2. Create a host manifest

macOS example:

```bash
mkdir -p qa/evidence/p3
python3 scripts/p3_real_host_gate.py init \
  --platform macos-arm64 \
  --commit "$(git rev-parse HEAD)" \
  --run-id "mac-arm64-001" \
  --output qa/evidence/p3/macos-arm64.json
```

Windows PowerShell example:

```powershell
New-Item -ItemType Directory -Force qa/evidence/p3 | Out-Null
$sha = (git rev-parse HEAD).Trim()
python scripts/p3_real_host_gate.py init `
  --platform windows-x86_64 `
  --commit $sha `
  --run-id windows-x64-001 `
  --output qa/evidence/p3/windows-x86_64.json
```

List the checks and their instructions:

```bash
python3 scripts/p3_real_host_gate.py catalog --platform macos-arm64
```

## 3. Record results

Record one result only after performing the catalog instruction against the exact candidate. A passing check requires at least one content-blind evidence reference.

```bash
python3 scripts/p3_real_host_gate.py record \
  --manifest qa/evidence/p3/macos-arm64.json \
  --check macos.accessibility_prompt_and_grant \
  --result pass \
  --evidence run=mac-arm64-001-accessibility \
  --note "grant state observed after returning from System Settings"
```

Use `fail` for a reproducible product failure and `blocked` when the required environment or approval is unavailable. Neither status is a pass. A required check cannot be `not-applicable`.

Reset an accidentally recorded result before rerunning it:

```bash
python3 scripts/p3_real_host_gate.py reset \
  --manifest qa/evidence/p3/macos-arm64.json \
  --check macos.accessibility_prompt_and_grant
```

Do not use a note to paste test content or logs. Store screenshots and diagnostic files in an approved restricted location, then reference only an opaque run ID, HTTPS URL, repository-relative path, or digest.

## 4. Verify each host and the complete set

During execution, validate a partial manifest without claiming completion:

```bash
python3 scripts/p3_real_host_gate.py verify qa/evidence/p3/macos-arm64.json
```

At host sign-off, require every mandatory check to pass:

```bash
python3 scripts/p3_real_host_gate.py verify \
  --require-complete \
  --expected-commit "$(git rev-parse HEAD)" \
  qa/evidence/p3/macos-arm64.json
```

After all three real-host manifests are complete:

```bash
python3 scripts/p3_real_host_gate.py verify-set \
  --expected-commit "$(git rev-parse HEAD)" \
  qa/evidence/p3/windows-x86_64.json \
  qa/evidence/p3/macos-arm64.json \
  qa/evidence/p3/macos-x86_64.json
```

Generate a review summary that omits evidence values:

```bash
python3 scripts/p3_real_host_gate.py report \
  qa/evidence/p3/windows-x86_64.json \
  qa/evidence/p3/macos-arm64.json \
  qa/evidence/p3/macos-x86_64.json \
  --output qa/evidence/p3/summary.md
```

The `P3 Real Host Evidence Gate` workflow validates the tool contract on pull requests. Its manual dispatch validates committed manifests against an exact SHA. Enable `require_release` only when the signed/notarized manifest is present.

## 5. Run unsigned release preflight

Before protected credentials are made available, verify repository provenance and release inputs:

```bash
python3 scripts/p3_release_preflight.py \
  --repo . \
  --commit "$(git rev-parse HEAD)" \
  --require-clean \
  --json-output /tmp/cliptype-p3-preflight.json
```

For an unsigned Universal 2 candidate on macOS:

```bash
python3 scripts/p3_release_preflight.py \
  --repo . \
  --commit "$(git rev-parse HEAD)" \
  --require-clean \
  --app dist/ClipType.app \
  --require-bundle \
  --require-universal2 \
  --artifact dist/ClipType-macos-universal.zip \
  --artifact dist/ClipType-macos-universal.dmg \
  --require-artifacts \
  --json-output /tmp/cliptype-p3-unsigned-preflight.json
```

The command emits metadata and SHA-256 values only. It does not search for or print signing secrets.

## 6. Protected signing and notarization gate

Run signing/notarization only through the protected release environment and existing release workflow. The final local verification must require every release property:

```bash
python3 scripts/p3_release_preflight.py \
  --repo . \
  --commit "$(git rev-parse HEAD)" \
  --require-clean \
  --app dist/ClipType.app \
  --require-bundle \
  --require-universal2 \
  --require-signing \
  --require-notarization \
  --artifact dist/ClipType-macos-universal.zip \
  --artifact dist/ClipType-macos-universal.dmg \
  --require-artifacts \
  --json-output /tmp/cliptype-p3-release-preflight.json
```

Then create `release-macos.json`, record the release checks with opaque submission/run references and artifact digests, and run the full gate:

```bash
python3 scripts/p3_real_host_gate.py verify-set \
  --require-release \
  --expected-commit "$(git rev-parse HEAD)" \
  qa/evidence/p3/windows-x86_64.json \
  qa/evidence/p3/macos-arm64.json \
  qa/evidence/p3/macos-x86_64.json \
  qa/evidence/p3/release-macos.json
```

## Promotion rule

The implementation PR can leave Draft only when all of the following are true:

1. Permanent hosted checks pass on the exact PR head.
2. The three real-host manifests are complete for that same commit.
3. An independent reviewer has checked architecture, privacy, unsafe/FFI boundaries, and the evidence references.
4. No unresolved `fail` or `blocked` result remains.

Signing, notarization, tagging, merging, and public release are separate authorized actions. A successful unsigned preflight or real-host gate must never be represented as a completed Apple release.
