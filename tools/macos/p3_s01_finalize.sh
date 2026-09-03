#!/usr/bin/env bash
set -euo pipefail

[[ $# -eq 1 ]] || {
  echo "usage: tools/macos/p3_s01_finalize.sh <evidence-dir>" >&2
  exit 2
}

EVIDENCE_DIR="$1"
MATRIX="$EVIDENCE_DIR/case-matrix.tsv"
RESULTS="$EVIDENCE_DIR/results.tsv"
ENVIRONMENT="$EVIDENCE_DIR/environment.tsv"
[[ -f "$MATRIX" && -f "$RESULTS" && -f "$ENVIRONMENT" ]] || {
  echo "evidence workspace is incomplete" >&2
  exit 2
}

REPORT="$EVIDENCE_DIR/P3_MACOS_NATIVE_SPIKE.md"
SUMMARY="$EVIDENCE_DIR/summary.tsv"

pass_count="$(awk -F '\t' 'NR > 1 && $2 == "PASS" { count++ } END { print count+0 }' "$RESULTS")"
limitation_count="$(awk -F '\t' 'NR > 1 && $2 == "LIMITATION" { count++ } END { print count+0 }' "$RESULTS")"
fail_count="$(awk -F '\t' 'NR > 1 && $2 == "FAIL" { count++ } END { print count+0 }' "$RESULTS")"
blocked_count="$(awk -F '\t' 'NR > 1 && $2 == "BLOCKED" { count++ } END { print count+0 }' "$RESULTS")"
not_run_count="$(awk -F '\t' 'NR > 1 && $2 == "NOT_RUN" { count++ } END { print count+0 }' "$RESULTS")"
required_unresolved="$(awk -F '\t' '
  NR == FNR && FNR > 1 && $3 == "yes" { required[$1]=1; next }
  FNR > 1 && required[$1] && $2 != "PASS" && $2 != "LIMITATION" { count++ }
  END { print count+0 }
' "$MATRIX" "$RESULTS")"
review_pass="$(awk -F '\t' 'NR > 1 && $1 == "FINAL-01" && $2 == "PASS" { print "true"; found=1 } END { if (!found) print "false" }' "$RESULTS")"

if [[ "$required_unresolved" -eq 0 && "$fail_count" -eq 0 && "$blocked_count" -eq 0 && "$not_run_count" -eq 0 && "$review_pass" == true ]]; then
  verdict="YES"
  gate="MACOS_NATIVE_MECHANISMS_READY"
else
  verdict="NO"
  gate="MACOS_NATIVE_MECHANISMS_NOT_READY"
fi

{
  printf 'key\tvalue\n'
  printf 'pass\t%s\n' "$pass_count"
  printf 'limitations\t%s\n' "$limitation_count"
  printf 'fail\t%s\n' "$fail_count"
  printf 'blocked\t%s\n' "$blocked_count"
  printf 'not_run\t%s\n' "$not_run_count"
  printf 'required_unresolved\t%s\n' "$required_unresolved"
  printf 'independent_review_pass\t%s\n' "$review_pass"
  printf 'production_adapters_may_proceed\t%s\n' "$verdict"
  printf 'gate\t%s\n' "$gate"
} > "$SUMMARY"

value() {
  awk -F '\t' -v key="$1" 'NR > 1 && $1 == key { print $2; exit }' "$ENVIRONMENT"
}

cat > "$REPORT" <<EOF_REPORT
# P3 macOS Native Mechanism Spike

## 1. Final recommendation

**P3 macOS production adapters may proceed: $verdict**

**Gate:** \`$gate\`

This report is generated from the exact evidence workspace. Hosted CI compilation is not counted as physical interactive evidence. A YES result requires every required case to be PASS or an explicitly documented LIMITATION, zero failed/blocked/not-run cases, and an independent-review PASS.

## 2. Tested source and environment

- Product candidate SHA: \`$(value product_candidate_sha)\`
- Validation tooling SHA: \`$(value validation_tooling_sha)\`
- macOS: \`$(value macos_version)\` build \`$(value macos_build)\`
- Architecture: \`$(value architecture)\`
- Hardware model: \`$(value hardware_model)\`
- Console session present: \`$(value console_session_present)\`
- Xcode: \`$(value xcode)\`
- macOS SDK: \`$(value macos_sdk)\`
- Rust: \`$(value rustc)\`
- App binary SHA-256: \`$(value app_binary_sha256)\`
- App architectures: \`$(value app_architectures)\`
- Bundle identifier: \`$(value bundle_identifier)\`
- Bundle version: \`$(value bundle_short_version)\` (build \`$(value bundle_build_version)\`)
- Minimum system version: \`$(value minimum_system_version)\`
- Code-sign verification: \`$(value codesign_verify)\`
- Gatekeeper assessment: \`$(value spctl_assessment)\`

## 3. Result counts

- PASS: \`$pass_count\`
- LIMITATION: \`$limitation_count\`
- FAIL: \`$fail_count\`
- BLOCKED: \`$blocked_count\`
- NOT RUN: \`$not_run_count\`
- Required unresolved: \`$required_unresolved\`
- Independent review: \`$review_pass\`

## 4. Observed results

| Case | Category | Required | Result | Detail code | Measurement |
|---|---|---|---|---|---|
EOF_REPORT

awk -F '\t' '
  NR == FNR && FNR > 1 { category[$1]=$2; required[$1]=$3; description[$1]=$4; order[++count]=$1; next }
  FNR > 1 { status[$1]=$2; detail[$1]=$3; measurement[$1]=$4 }
  END {
    for (i=1; i<=count; i++) {
      id=order[i]
      printf "| `%s` | %s | %s | %s | `%s` | `%s` |\n", id, category[id], required[id], status[id], detail[id], measurement[id]
    }
  }
' "$MATRIX" "$RESULTS" >> "$REPORT"

cat >> "$REPORT" <<'EOF_REPORT'

The table records result categories and bounded measurements only. Clipboard text, injected text, focused-field content, window titles, usernames, native handles, and privacy sentinels are excluded.

## 5. Inferences

- A successful native build proves source compatibility with that SDK and architecture; it does not prove unlocked-desktop input behavior.
- A successful global-hotkey registration probe proves only OS registration availability at that moment. Application-local shortcuts and hook-based tools may still conflict.
- Frontmost-process identity is weaker than focused Accessibility-element identity. Detailed evidence that later degrades must fail closed.
- An ad-hoc signature is suitable only for a clearly labelled development candidate. It is not a Developer ID or notarization result.

## 6. Recommended production contract

- Clipboard: one bounded `NSPasteboard.general` text read, content-blind `changeCount`, no history/write/clear/restore in the product.
- Keyboard: one semantic action per bounded Core Graphics dispatch; no retry after partial or unknown progress.
- Paste: recheck the expected pasteboard revision immediately before one balanced Command-V chord.
- Destination: capture frontmost process plus focused Accessibility element when authorized; stop on change, disappearance, ambiguity, or post-start evidence degradation.
- Permission: explicit system-controlled prompt/remediation only; represent initial, not-granted, granted, and revoked states without claiming the OS exposes a durable denial history.
- Hotkeys: register Trigger and Cancel as one transaction; occupied candidates must leave the complete previous pair active.
- Shell: AppKit status-item and permission operations stay on the main thread; the injection worker remains bounded and separate.
- Login item: only the app-owned `SMAppService.mainApp` registration is permitted.

## 7. Thread ownership

| Surface | Owner |
|---|---|
| `NSApplication`, `NSStatusItem`, `NSMenu`, permission prompt, settings presentation | main AppKit thread |
| Carbon global-hotkey registration and replacement | owning application/main run-loop thread |
| Clipboard/target/modifier snapshots | bounded adapter calls from the coordinator worker |
| Core Graphics dispatch | bounded coordinator worker action |
| Slint settings callbacks | UI event loop, forwarded through typed channels |
| Shutdown | main shell requests cancellation, then waits boundedly |

## 8. Permission matrix

| Observation | Product state | Input behavior |
|---|---|---|
| No explicit onboarding action observed in this process | Not requested / initial unknown boundary | synthetic input unavailable |
| Explicit request/settings action made, trust still false | Not granted | synthetic input unavailable; fixed remediation only |
| `AXIsProcessTrusted` true | Granted | input capability may be available, subject to Secure Event Input and target evidence |
| Trust was observed true and later becomes false | Revoked while running | stop/fail closed; do not reprompt automatically |
| Native query fails or cannot be interpreted | Unknown | fail closed |

## 9. Proposed production bounds

- Clipboard hard limit: 8 MiB.
- One original scalar, line break, Tab, wrong key, Backspace, or corrected key per paced action.
- No backend switch after a session starts.
- No destination adoption or refocus.
- No automatic Accessibility prompt loop.
- No public macOS support claim from hosted runners alone.

## 10. Limitations and unresolved questions

Cases marked LIMITATION are retained here and require explicit reviewer acceptance. Typical unresolved boundaries include Accessibility identity stability across rebuild/signature changes, focused-element identity stability across applications, Carbon shortcut conflicts, login-item behavior across a real logout/login cycle, same-render-host fields, Secure Event Input, and Intel runtime availability.

## 11. Privacy statement

The public report is content-free by construction. A generated privacy sentinel must be absent from evidence, diagnostics, settings, crash output, and distributable files before PRIV-01 can pass.
EOF_REPORT

printf 'report=%s\n' "$REPORT"
printf 'production_adapters_may_proceed=%s\n' "$verdict"
[[ "$verdict" == "YES" ]]
