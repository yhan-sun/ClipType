#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage: tools/macos/p3_s01_prepare.sh <40-char-product-sha> <ClipType.app> [evidence-dir]

Prepares a content-free P3-S01 evidence workspace on an interactive Mac.
The application bundle must be the exact candidate under test. The current
checkout may differ from the product SHA only in P3-S01 validation tooling and
documentation.
USAGE
  exit 2
}

[[ $# -ge 2 && $# -le 3 ]] || usage
PRODUCT_SHA="$1"
APP_INPUT="$2"
EVIDENCE_DIR="${3:-p3-s01-evidence-${PRODUCT_SHA:0:12}}"

[[ "$(uname -s)" == "Darwin" ]] || { echo "P3-S01 requires macOS" >&2; exit 2; }
[[ "$PRODUCT_SHA" =~ ^[0-9a-f]{40}$ ]] || { echo "invalid product SHA" >&2; exit 2; }
[[ -d "$APP_INPUT/Contents" ]] || { echo "ClipType.app bundle not found" >&2; exit 2; }

git rev-parse --verify "$PRODUCT_SHA^{commit}" >/dev/null
git merge-base --is-ancestor "$PRODUCT_SHA" HEAD

unexpected="$({
  git diff --name-only "$PRODUCT_SHA"..HEAD
} | grep -Ev '^(tools/macos/|docs/research/P3_MACOS_NATIVE_SPIKE\.md$|docs/testing/P3_MACOS_INTERACTIVE_RUNBOOK\.md$|\.github/workflows/p3-s01-macos-validation\.yml$|\.github/workflows/p3-s01-source-export\.yml$)' || true)"
[[ -z "$unexpected" ]] || {
  echo "checkout contains product changes after the declared candidate SHA:" >&2
  printf '%s\n' "$unexpected" >&2
  exit 1
}

umask 077
mkdir -p "$EVIDENCE_DIR/bin" "$EVIDENCE_DIR/probes" "$EVIDENCE_DIR/screenshots"
cp tools/macos/p3_s01_cases.tsv "$EVIDENCE_DIR/case-matrix.tsv"
awk -F '\t' 'BEGIN { OFS="\t"; print "case_id","status","detail_code","measurement","recorded_at_utc" } NR > 1 { print $1,"NOT_RUN","not_run","","" }' \
  tools/macos/p3_s01_cases.tsv > "$EVIDENCE_DIR/results.tsv"

TOOLING_SHA="$(git rev-parse HEAD)"
APP="$(cd "$(dirname "$APP_INPUT")" && pwd -P)/$(basename "$APP_INPUT")"
BINARY="$APP/Contents/MacOS/ClipType"
[[ -x "$BINARY" ]] || { echo "ClipType executable is missing" >&2; exit 1; }

console_user="$(stat -f '%Su' /dev/console 2>/dev/null || true)"
if [[ -n "$console_user" && "$console_user" != "root" && "$console_user" != "loginwindow" ]]; then
  console_session=true
else
  console_session=false
fi

product_version="$(sw_vers -productVersion)"
product_build="$(sw_vers -buildVersion)"
architecture="$(uname -m)"
hardware_model="$(sysctl -n hw.model 2>/dev/null || printf unknown)"
xcode_version="$(xcodebuild -version 2>/dev/null | tr '\n' ';' | sed 's/;$//' || printf unavailable)"
sdk_version="$(xcrun --sdk macosx --show-sdk-version 2>/dev/null || printf unavailable)"
rust_version="$(rustc -V 2>/dev/null || printf unavailable)"
cargo_version="$(cargo -V 2>/dev/null || printf unavailable)"
app_binary_sha256="$(shasum -a 256 "$BINARY" | awk '{print $1}')"
app_architectures="$(lipo -archs "$BINARY" | tr ' ' ',')"
bundle_identifier="$(plutil -extract CFBundleIdentifier raw -o - "$APP/Contents/Info.plist")"
bundle_short_version="$(plutil -extract CFBundleShortVersionString raw -o - "$APP/Contents/Info.plist")"
bundle_build_version="$(plutil -extract CFBundleVersion raw -o - "$APP/Contents/Info.plist")"
minimum_system_version="$(plutil -extract LSMinimumSystemVersion raw -o - "$APP/Contents/Info.plist")"
ls_ui_element="$(plutil -extract LSUIElement raw -o - "$APP/Contents/Info.plist")"

codesign_status=failed
if codesign --verify --deep --strict --verbose=2 "$APP" >"$EVIDENCE_DIR/probes/codesign-verify.txt" 2>&1; then
  codesign_status=pass
fi
spctl_status=rejected
if spctl --assess --type execute --verbose=2 "$APP" >"$EVIDENCE_DIR/probes/spctl-assess.txt" 2>&1; then
  spctl_status=accepted
fi

{
  printf 'key\tvalue\n'
  printf 'schema\t%s\n' 1
  printf 'product_candidate_sha\t%s\n' "$PRODUCT_SHA"
  printf 'validation_tooling_sha\t%s\n' "$TOOLING_SHA"
  printf 'macos_version\t%s\n' "$product_version"
  printf 'macos_build\t%s\n' "$product_build"
  printf 'architecture\t%s\n' "$architecture"
  printf 'hardware_model\t%s\n' "$hardware_model"
  printf 'console_session_present\t%s\n' "$console_session"
  printf 'xcode\t%s\n' "$xcode_version"
  printf 'macos_sdk\t%s\n' "$sdk_version"
  printf 'rustc\t%s\n' "$rust_version"
  printf 'cargo\t%s\n' "$cargo_version"
  printf 'bundle_identifier\t%s\n' "$bundle_identifier"
  printf 'bundle_short_version\t%s\n' "$bundle_short_version"
  printf 'bundle_build_version\t%s\n' "$bundle_build_version"
  printf 'minimum_system_version\t%s\n' "$minimum_system_version"
  printf 'ls_ui_element\t%s\n' "$ls_ui_element"
  printf 'app_binary_sha256\t%s\n' "$app_binary_sha256"
  printf 'app_architectures\t%s\n' "$app_architectures"
  printf 'codesign_verify\t%s\n' "$codesign_status"
  printf 'spctl_assessment\t%s\n' "$spctl_status"
} > "$EVIDENCE_DIR/environment.tsv"

export CLIPTYPE_SOURCE_SHA="$PRODUCT_SHA"
cargo build --release --locked -p cliptype-macos --example p3_s01_probe
PROBE="target/release/examples/p3_s01_probe"
cp "$PROBE" "$EVIDENCE_DIR/bin/"
"$PROBE" snapshot > "$EVIDENCE_DIR/probes/runtime-snapshot.txt"
"$PROBE" status-item-smoke > "$EVIDENCE_DIR/probes/status-item-smoke.txt"
if "$PROBE" hotkey-cycle > "$EVIDENCE_DIR/probes/hotkey-cycle.txt" 2>&1; then
  printf 'hotkey_cycle\tpass\n' > "$EVIDENCE_DIR/probes/automated-status.tsv"
else
  printf 'hotkey_cycle\tfail\n' > "$EVIDENCE_DIR/probes/automated-status.tsv"
fi

swiftc tools/macos/p3_s01_target.swift -framework AppKit -o "$EVIDENCE_DIR/bin/p3_s01_target"

cat > "$EVIDENCE_DIR/README.txt" <<EOF_README
ClipType P3-S01 interactive evidence workspace

Product candidate SHA: $PRODUCT_SHA
Validation tooling SHA: $TOOLING_SHA

1. Read docs/testing/P3_MACOS_INTERACTIVE_RUNBOOK.md.
2. Launch the controlled target with:
   P3_S01_RESULTS_JSONL="$EVIDENCE_DIR/target-results.jsonl" \\
     "$EVIDENCE_DIR/bin/p3_s01_target"
3. Launch the exact app bundle from its stable installed path.
4. Record each manual case with:
   tools/macos/p3_s01_record.sh "$EVIDENCE_DIR" CASE-ID PASS detail_code [measurement]
5. Run the privacy sentinel procedure from the runbook.
6. Finalize with:
   tools/macos/p3_s01_finalize.sh "$EVIDENCE_DIR"

Never paste real user data into the controlled target. Public evidence must not
contain clipboard text, focused-field content, window titles, usernames, native
handles, or the generated privacy sentinel.
EOF_README

printf 'P3-S01 evidence workspace prepared: %s\n' "$EVIDENCE_DIR"
printf 'product_candidate_sha=%s\n' "$PRODUCT_SHA"
printf 'validation_tooling_sha=%s\n' "$TOOLING_SHA"
