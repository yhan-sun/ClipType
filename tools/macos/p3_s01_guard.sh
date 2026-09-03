#!/usr/bin/env bash
set -euo pipefail

# This guard compares every validation commit with the fixed product candidate.
PRODUCT_SHA="${P3_PRODUCT_SHA:?P3_PRODUCT_SHA is required}"
git cat-file -e "${PRODUCT_SHA}^{commit}"

unexpected=()
while IFS= read -r path; do
  case "$path" in
    .github/workflows/p3-s01-macos-validation.yml | \
    crates/cliptype-macos/examples/p3_s01_probe.rs | \
    docs/research/P3_MACOS_NATIVE_SPIKE.md | \
    docs/testing/P3_MACOS_INTERACTIVE_RUNBOOK.md | \
    tools/macos/*)
      ;;
    *)
      unexpected+=("$path")
      ;;
  esac
done < <(git diff --name-only "${PRODUCT_SHA}"..HEAD)

if (( ${#unexpected[@]} != 0 )); then
  printf 'validation branch changed product paths after %s:\n' "$PRODUCT_SHA" >&2
  printf '  %s\n' "${unexpected[@]}" >&2
  exit 1
fi

printf 'product_candidate_sha=%s\n' "$PRODUCT_SHA"
printf 'validation_tooling_sha=%s\n' "$(git rev-parse HEAD)"
printf 'product_path_drift=absent\n'
