#!/usr/bin/env bash
set -euo pipefail

[[ $# -ge 4 && $# -le 5 ]] || {
  echo "usage: tools/macos/p3_s01_record.sh <evidence-dir> <case-id> <PASS|FAIL|LIMITATION|BLOCKED|NOT_RUN> <detail-code> [measurement]" >&2
  exit 2
}

EVIDENCE_DIR="$1"
CASE_ID="$2"
STATUS="$3"
DETAIL_CODE="$4"
MEASUREMENT="${5:-}"
MATRIX="$EVIDENCE_DIR/case-matrix.tsv"
RESULTS="$EVIDENCE_DIR/results.tsv"

[[ -f "$MATRIX" && -f "$RESULTS" ]] || { echo "evidence workspace is not initialized" >&2; exit 2; }
case "$STATUS" in
  PASS|FAIL|LIMITATION|BLOCKED|NOT_RUN) ;;
  *) echo "invalid status" >&2; exit 2 ;;
esac
[[ "$CASE_ID" =~ ^[A-Z]+-[0-9]{2}$ ]] || { echo "invalid case id" >&2; exit 2; }
awk -F '\t' -v id="$CASE_ID" 'NR > 1 && $1 == id { found=1 } END { exit found ? 0 : 1 }' "$MATRIX" || {
  echo "unknown case id" >&2
  exit 2
}
[[ "$DETAIL_CODE" =~ ^[a-z0-9][a-z0-9_.:-]{0,79}$ ]] || { echo "detail code must be content-free" >&2; exit 2; }
[[ -z "$MEASUREMENT" || "$MEASUREMENT" =~ ^[A-Za-z0-9][A-Za-z0-9_.:+,%/=-]{0,79}$ ]] || {
  echo "measurement must be a short content-free token" >&2
  exit 2
}

NOW="$(date -u +'%Y-%m-%dT%H:%M:%SZ')"
TEMP="$RESULTS.tmp"
awk -F '\t' -v OFS='\t' -v id="$CASE_ID" -v status="$STATUS" -v detail="$DETAIL_CODE" -v measurement="$MEASUREMENT" -v now="$NOW" '
  NR == 1 { print; next }
  $1 == id { print $1, status, detail, measurement, now; updated=1; next }
  { print }
  END { if (!updated) exit 3 }
' "$RESULTS" > "$TEMP"
mv "$TEMP" "$RESULTS"
printf '%s\t%s\t%s\t%s\n' "$CASE_ID" "$STATUS" "$DETAIL_CODE" "$MEASUREMENT"
