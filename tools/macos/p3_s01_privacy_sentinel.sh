#!/usr/bin/env bash
set -euo pipefail

[[ $# -eq 2 ]] || {
  echo "usage: tools/macos/p3_s01_privacy_sentinel.sh <evidence-dir> <ClipType.app>" >&2
  exit 2
}

EVIDENCE_DIR="$1"
APP="$2"
[[ "$(uname -s)" == "Darwin" ]] || { echo "macOS required" >&2; exit 2; }
[[ -f "$EVIDENCE_DIR/results.tsv" ]] || { echo "evidence workspace is not initialized" >&2; exit 2; }
[[ -d "$APP/Contents" ]] || { echo "ClipType.app not found" >&2; exit 2; }

umask 077
MARKER_FILE="$(mktemp -t cliptype-p3-s01-sentinel)"
cleanup() {
  printf '' | pbcopy || true
  rm -f "$MARKER_FILE"
}
trap cleanup EXIT

uuid="$(uuidgen | tr -d '-' | tr '[:lower:]' '[:upper:]')"
printf 'CLIPTYPE_P3_S01_PRIVACY_%s' "$uuid" > "$MARKER_FILE"
pbcopy < "$MARKER_FILE"

cat <<'INSTRUCTIONS'
A generated synthetic privacy marker is now on the clipboard.

1. Launch the exact ClipType.app candidate from its stable path.
2. Launch the controlled target from the prepared evidence workspace.
3. Focus an empty controlled field and invoke the physical Trigger shortcut once.
4. Confirm only the controlled target received the marker.
5. Return here and press Enter. Do not paste the marker into terminals, notes, issue comments, or screenshots.
INSTRUCTIONS
read -r _

clipboard_unchanged=false
if cmp -s "$MARKER_FILE" <(pbpaste); then
  clipboard_unchanged=true
fi

roots=(
  "$EVIDENCE_DIR"
  "$APP"
  "$HOME/Library/Application Support/ClipType"
  "$HOME/Library/Logs/ClipType"
  "$HOME/Library/Logs/DiagnosticReports"
)
match_count=0
while IFS= read -r -d '' root; do
  while IFS= read -r -d '' file; do
    if LC_ALL=C grep -a -F -q -f "$MARKER_FILE" "$file" 2>/dev/null; then
      match_count=$((match_count + 1))
    fi
  done < <(find "$root" -type f -print0 2>/dev/null)
done < <(printf '%s\0' "${roots[@]}" | while IFS= read -r -d '' candidate; do [[ -e "$candidate" ]] && printf '%s\0' "$candidate"; done)

if [[ "$clipboard_unchanged" == true ]]; then
  tools/macos/p3_s01_record.sh "$EVIDENCE_DIR" CLIP-04 PASS clipboard_unchanged
else
  tools/macos/p3_s01_record.sh "$EVIDENCE_DIR" CLIP-04 FAIL clipboard_changed
fi

if [[ "$match_count" -eq 0 ]]; then
  tools/macos/p3_s01_record.sh "$EVIDENCE_DIR" PRIV-01 PASS sentinel_absent "matches=0"
  printf 'privacy_sentinel=absent\n'
else
  tools/macos/p3_s01_record.sh "$EVIDENCE_DIR" PRIV-01 FAIL sentinel_found "matches=$match_count"
  printf 'privacy_sentinel=found matches=%d\n' "$match_count"
  exit 1
fi
