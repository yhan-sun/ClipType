#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
cd "$ROOT"

for script in tools/macos/p3_s01_*.sh; do
  bash -n "$script"
done

header="$(head -n 1 tools/macos/p3_s01_cases.tsv)"
[[ "$header" == $'case_id\tcategory\trequired\tdescription' ]] || {
  echo "case matrix must be a real tab-separated file" >&2
  exit 1
}

work="$(mktemp -d -t cliptype-p3-s01-self-test.XXXXXX)"
cleanup() {
  rm -rf "$work"
}
trap cleanup EXIT

initialize_workspace() {
  local destination="$1"
  mkdir -p "$destination"
  cp tools/macos/p3_s01_cases.tsv "$destination/case-matrix.tsv"
  awk -F '\t' 'BEGIN {
      OFS="\t"
      print "case_id", "status", "detail_code", "measurement", "recorded_at_utc"
    }
    NR > 1 { print $1, "NOT_RUN", "not_run", "", "" }
  ' tools/macos/p3_s01_cases.tsv > "$destination/results.tsv"
  {
    printf 'key\tvalue\n'
    printf 'product_candidate_sha\t%s\n' '0000000000000000000000000000000000000000'
    printf 'validation_tooling_sha\t%s\n' 'self-test'
    printf 'macos_version\t%s\n' 'synthetic'
    printf 'macos_build\t%s\n' 'synthetic'
    printf 'architecture\t%s\n' 'synthetic'
    printf 'hardware_model\t%s\n' 'synthetic'
    printf 'console_session_present\t%s\n' 'false'
    printf 'xcode\t%s\n' 'synthetic'
    printf 'macos_sdk\t%s\n' 'synthetic'
    printf 'rustc\t%s\n' 'synthetic'
    printf 'app_binary_sha256\t%s\n' 'synthetic'
    printf 'app_architectures\t%s\n' 'synthetic'
    printf 'bundle_identifier\t%s\n' 'synthetic'
    printf 'bundle_short_version\t%s\n' 'synthetic'
    printf 'bundle_build_version\t%s\n' 'synthetic'
    printf 'minimum_system_version\t%s\n' 'synthetic'
    printf 'codesign_verify\t%s\n' 'synthetic'
    printf 'spctl_assessment\t%s\n' 'synthetic'
  } > "$destination/environment.tsv"
}

negative="$work/negative"
positive="$work/positive"
initialize_workspace "$negative"
initialize_workspace "$positive"

set +e
tools/macos/p3_s01_finalize.sh "$negative" > "$work/negative.out" 2> "$work/negative.err"
negative_exit=$?
set -e
[[ "$negative_exit" -eq 1 ]]
grep -q '^production_adapters_may_proceed=NO$' "$work/negative.out"
awk -F '\t' 'NR > 1 && $1 == "production_adapters_may_proceed" && $2 == "NO" { found=1 } END { exit found ? 0 : 1 }' "$negative/summary.tsv"
grep -q 'P3 macOS production adapters may proceed: NO' "$negative/P3_MACOS_NATIVE_SPIKE.md"

while IFS=$'\t' read -r case_id _category _required _description; do
  [[ "$case_id" == "case_id" ]] && continue
  tools/macos/p3_s01_record.sh \
    "$positive" "$case_id" PASS synthetic_pass count=1 > /dev/null
done < tools/macos/p3_s01_cases.tsv

tools/macos/p3_s01_finalize.sh "$positive" > "$work/positive.out"
grep -q '^production_adapters_may_proceed=YES$' "$work/positive.out"
awk -F '\t' 'NR > 1 && $1 == "production_adapters_may_proceed" && $2 == "YES" { found=1 } END { exit found ? 0 : 1 }' "$positive/summary.tsv"
grep -q 'P3 macOS production adapters may proceed: YES' "$positive/P3_MACOS_NATIVE_SPIKE.md"

printf 'p3_s01_self_test=pass\n'
