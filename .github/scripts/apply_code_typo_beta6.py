#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def replace_once(rel: str, old: str, new: str) -> None:
    path = ROOT / rel
    text = path.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{rel}: expected exactly one replacement site, found {count}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


def replace_all_if_present(rel: str, old: str, new: str) -> None:
    path = ROOT / rel
    text = path.read_text(encoding="utf-8")
    if old not in text:
        return
    path.write_text(text.replace(old, new), encoding="utf-8")


# 1. Code mode must honor corrected-typo probability for source Atom actions.
replace_once(
    "crates/cliptype-app/src/coordinator.rs",
    '''    while let Some(action) = queue.pop_front() {
        let action = match action {
            CodeAction::Atom(atom) => KeyboardAction::Atom(atom),
            CodeAction::CursorRight => KeyboardAction::CursorRight,
            CodeAction::CursorRightToLineEnd => KeyboardAction::CursorRightToLineEnd,
        };

        if matches!(
''',
    '''    while let Some(code_action) = queue.pop_front() {
        let action = match code_action {
            CodeAction::Atom(atom) => KeyboardAction::Atom(atom),
            CodeAction::CursorRight => KeyboardAction::CursorRight,
            CodeAction::CursorRightToLineEnd => KeyboardAction::CursorRightToLineEnd,
        };

        // Code mode uses the same corrected-typo setting as Keyboard mode,
        // but only for source Atom actions. Navigation actions represent
        // editor-generated closers and must never be humanized into typos.
        // The Code-specific typo helper rejects temporary characters that can
        // themselves trigger editor auto-pair, quote, or comment behavior.
        if let KeyboardAction::Atom(atom) = action {
            if let Some(wrong) = code_adjacent_typo(
                atom,
                context.config.typo_probability_percent,
                &mut random,
            ) {
                if let Err(outcome) = dispatch_timed_action(
                    context,
                    plan.config(),
                    KeyboardAction::Atom(wrong),
                    &mut random,
                ) {
                    return SessionCompletion::Finished(outcome);
                }
                if let Err(outcome) = dispatch_timed_action(
                    context,
                    plan.config(),
                    KeyboardAction::Backspace,
                    &mut random,
                ) {
                    return SessionCompletion::Finished(outcome);
                }
            }
        }

        if matches!(
''',
)

# 2. Split probability from candidate selection and add a Code-safe candidate policy.
replace_once(
    "crates/cliptype-app/src/coordinator.rs",
    '''fn adjacent_typo(
    atom: TextAtom,
    probability_percent: u8,
    random: &mut TypingRandom,
) -> Option<TextAtom> {
    if probability_percent == 0 || random.below(100) >= u64::from(probability_percent) {
        return None;
    }
    let TextAtom::Scalar(value) = atom else {
''',
    '''fn adjacent_typo(
    atom: TextAtom,
    probability_percent: u8,
    random: &mut TypingRandom,
) -> Option<TextAtom> {
    if !typo_probability_hit(probability_percent, random) {
        return None;
    }
    adjacent_typo_candidate(atom, random)
}

fn code_adjacent_typo(
    atom: TextAtom,
    probability_percent: u8,
    random: &mut TypingRandom,
) -> Option<TextAtom> {
    if !typo_probability_hit(probability_percent, random) {
        return None;
    }

    // Rejection-sample the existing physical-neighbour map so Code mode never
    // types a temporary auto-pair opener/closer, quote, or slash. Those keys can
    // mutate editor state before Backspace and invalidate the planned cursor
    // navigation. A right-bracket source has only structural neighbours in the
    // shared map, so use its physical backslash neighbour as the safe fallback.
    for _ in 0..16 {
        let candidate = adjacent_typo_candidate(atom, random)?;
        if candidate
            .exposed_scalar()
            .is_some_and(code_typo_candidate_is_safe)
        {
            return Some(candidate);
        }
    }

    match atom.exposed_scalar() {
        Some(']') => Some(TextAtom::Scalar('\\\\')),
        _ => None,
    }
}

fn typo_probability_hit(probability_percent: u8, random: &mut TypingRandom) -> bool {
    probability_percent != 0 && random.below(100) < u64::from(probability_percent)
}

fn code_typo_candidate_is_safe(value: char) -> bool {
    value.is_ascii_alphanumeric()
        || matches!(value, '-' | '=' | ';' | ',' | '.' | '\\\\')
}

fn adjacent_typo_candidate(atom: TextAtom, random: &mut TypingRandom) -> Option<TextAtom> {
    let TextAtom::Scalar(value) = atom else {
''',
)

# 3. Unit regressions for Code-safe typo generation.
replace_once(
    "crates/cliptype-app/src/coordinator.rs",
    '''    use super::{TypingRandom, adjacent_typo, jittered_delay};
''',
    '''    use super::{
        TypingRandom, adjacent_typo, code_adjacent_typo, code_typo_candidate_is_safe,
        jittered_delay,
    };
''',
)

replace_once(
    "crates/cliptype-app/src/coordinator.rs",
    '''    #[test]
    fn zero_probability_and_zero_jitter_are_exact() {
''',
    '''    #[test]
    fn code_typos_are_correctable_without_structural_editor_side_effects() {
        for source in ['a', 'p', '[', ']', '\\'', '/', '='] {
            for seed in 1..=128 {
                let mut random = TypingRandom::new(seed);
                if let Some(wrong) = code_adjacent_typo(TextAtom::Scalar(source), 100, &mut random)
                {
                    let scalar = wrong.exposed_scalar().expect("code typo is a scalar");
                    assert!(code_typo_candidate_is_safe(scalar));
                    assert!(!matches!(scalar, '(' | ')' | '{' | '}' | '[' | ']' | '\\'' | '"' | '/'));
                }
            }
        }

        let mut random = TypingRandom::new(7);
        assert!(code_adjacent_typo(TextAtom::Scalar('a'), 100, &mut random).is_some());
        let mut random = TypingRandom::new(7);
        assert!(code_adjacent_typo(TextAtom::Scalar(']'), 100, &mut random).is_some());
        let mut random = TypingRandom::new(7);
        assert_eq!(
            code_adjacent_typo(TextAtom::Scalar('你'), 100, &mut random),
            None
        );
    }

    #[test]
    fn zero_probability_and_zero_jitter_are_exact() {
''',
)

# 4. Product documentation: Code typo correction is now an explicit contract.
replace_once(
    "docs/PRODUCT.md",
    '''formatting semantics.\n\n### Auto\n''',
    '''formatting semantics. When corrected-typo probability is non-zero, Code
mode applies it only to source Atom actions as `wrong key -> Backspace ->
correct source atom`. Temporary wrong keys are restricted so they cannot be
brackets, quotes, or `/`, preventing the typo simulation itself from triggering
editor auto-pair or comment behavior. Cursor-navigation actions are never typo
simulated, and non-ASCII text still has no fabricated QWERTY typo.\n\n### Auto\n''',
)

# Top-level README omitted Code from the mode list; keep the public surface accurate.
replace_once(
    "README.md",
    '''- `clipboard` — verifies the current clipboard revision and sends one ordinary `Ctrl+V`; ClipType never rewrites or restores the clipboard.\n- `auto` — freezes one proven backend per session from Unicode shape, payload size, and available capabilities; non-ASCII text prefers guarded paste.\n''',
    '''- `clipboard` — verifies the current clipboard revision and sends one ordinary `Ctrl+V`; ClipType never rewrites or restores the clipboard.\n- `code` — keyboard-only code-aware input with editor auto-pair/auto-indent navigation and safe corrected-typo simulation.\n- `auto` — freezes one proven backend per session from Unicode shape, payload size, and available capabilities; non-ASCII text prefers guarded paste.\n''',
)

# 5. Advance the immutable prerelease version.
replace_once("release/VERSION", "v0.2.0-beta.5\n", "v0.2.0-beta.6\n")
replace_once(
    "apps/cliptype-flutter/pubspec.yaml",
    "version: 0.2.0-beta.5+4\n",
    "version: 0.2.0-beta.6+5\n",
)

for rel in [
    "README.md",
    "apps/cliptype-flutter/README.md",
    "docs/README.md",
    "docs/COMPATIBILITY.md",
    "docs/PLATFORMS.md",
]:
    replace_all_if_present(rel, "v0.2.0-beta.5", "v0.2.0-beta.6")

release_notes = ROOT / "docs/releases/v0.2.0-beta.6.md"
if release_notes.exists():
    raise SystemExit("beta.6 release notes already exist")
release_notes.write_text(
    '''# ClipType v0.2.0-beta.6

`v0.2.0-beta.6` is a maintenance prerelease that brings corrected-typo simulation to Code mode without weakening code-aware editor navigation.

## Fix

- makes the configured corrected-typo probability apply to Code-mode source Atom actions, not only ordinary Keyboard mode;
- preserves the humanized sequence as `wrong key -> Backspace -> correct source atom` before the normal Code settle barrier;
- never typo-simulates `CursorRight` or `CursorRightToLineEnd` navigation actions;
- restricts temporary Code-mode wrong keys to editor-safe ASCII so they cannot themselves open brackets/quotes or start `//` / `/*` behavior;
- keeps non-ASCII text free of fabricated QWERTY typos;
- keeps Code mode keyboard-only with no paste fallback;
- documents Code mode in the top-level product-mode list and adds deterministic typo-safety regressions.

## Validation

Publication requires exact-head Rust, Flutter, Windows release/compatibility/controlled-input gates and the macOS arm64 build/package gate to pass. The corrected-typo tests are deterministic and verify that temporary Code-mode typo candidates cannot be structural pair/comment triggers.

## Platform boundary

Windows x86_64 remains primary. The macOS arm64 asset remains an ad-hoc-signed testing preview requiring Accessibility consent; it is not Developer ID signed, notarized, Universal 2, or a general macOS release.

## Rollback

Older tags and assets remain immutable.
''',
    encoding="utf-8",
)

# Bootstrap-only files must never land in the candidate commit.
for rel in [
    ".github/scripts/apply_code_typo_beta6.py",
    ".github/workflows/bootstrap-code-typo-beta6.yml",
    ".github/cliptype-code-typo-beta6-trigger",
]:
    path = ROOT / rel
    if path.exists():
        path.unlink()

print("beta6 code-mode corrected-typo patch applied")
