#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(relative: str, old: str, new: str) -> None:
    path = ROOT / relative
    text = path.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{relative}: expected one replacement, found {count}")
    path.write_text(text.replace(old, new), encoding="utf-8")


PRODUCT = "crates/cliptype-core/src/product.rs"
MACOS = "crates/cliptype-macos/src/native.rs"

replace_once(
    PRODUCT,
    """            CodeLexState::LineComment => {
                if matches!(atom, TextAtom::LineBreak) {
                    push_code_line_break(&mut actions, &mut pair_stack);
                    state = CodeLexState::Normal;
                    line_start = true;
                } else {
                    actions.push(CodeAction::Atom(atom));
                }
            }
""",
    """            CodeLexState::LineComment => {
                if matches!(atom, TextAtom::LineBreak) {
                    if let Some(line_closers) =
                        line_leading_matching_closers(atoms, index, &pair_stack)
                    {
                        if !line_closers.were_line_separated {
                            push_code_line_break(&mut actions, &mut pair_stack);
                        }
                        pair_stack
                            .truncate(pair_stack.len().saturating_sub(line_closers.pair_count));
                        actions.push(CodeAction::CursorRightToLineEnd);
                        state = CodeLexState::Normal;
                        line_start = false;
                        index = line_closers.end_index;
                        continue;
                    }

                    push_code_line_break(&mut actions, &mut pair_stack);
                    state = CodeLexState::Normal;
                    line_start = true;
                } else {
                    actions.push(CodeAction::Atom(atom));
                }
            }
""",
)

replace_once(
    PRODUCT,
    """                TextAtom::Scalar(value) if !quote.is_triple() && opening_pair(value).is_some() => {
                    // Some editors also auto-complete brackets typed inside a
                    // string. Keep those generated closers in the same logical
                    // stack so a later source closer or string boundary can
                    // consume them with CursorRight.
                    actions.push(CodeAction::Atom(atom));
                    pair_stack.push(Pair::Bracket {
                        closer: opening_pair(value).expect("checked above"),
                        line_separated: false,
                    });
                    line_start = false;
                }
                TextAtom::Scalar(value)
                    if !quote.is_triple()
                        && pair_stack
                            .last()
                            .is_some_and(|pair| pair.matches_bracket(value)) =>
                {
                    pair_stack.pop();
                    actions.push(CodeAction::CursorRight);
                    line_start = false;
                }
                TextAtom::Scalar(value) if !quote.is_triple() && value == quote.delimiter() => {
                    // A bracket such as the `{` in `"{"` can be auto-completed
                    // before the editor's generated quote. Move over all such
                    // generated closers first, then move over the quote.
                    flush_string_bracket_pairs(&mut actions, &mut pair_stack);
                    if pair_stack.last() == Some(&Pair::Quote(quote)) {
                        pair_stack.pop();
                        actions.push(CodeAction::CursorRight);
                    } else {
                        actions.push(CodeAction::Atom(atom));
                    }
                    state = CodeLexState::Normal;
                    line_start = false;
                }
""",
    """                TextAtom::Scalar(value) if !quote.is_triple() && value == quote.delimiter() => {
                    // Brackets inside strings are source literals. Only the
                    // editor-generated closing quote is skipped.
                    if pair_stack.last() == Some(&Pair::Quote(quote)) {
                        pair_stack.pop();
                        actions.push(CodeAction::CursorRight);
                    } else {
                        actions.push(CodeAction::Atom(atom));
                    }
                    state = CodeLexState::Normal;
                    line_start = false;
                }
""",
)

replace_once(
    PRODUCT,
    """fn flush_string_bracket_pairs(actions: &mut Vec<CodeAction>, pair_stack: &mut Vec<Pair>) {
    while matches!(pair_stack.last(), Some(Pair::Bracket { .. })) {
        pair_stack.pop();
        actions.push(CodeAction::CursorRight);
    }
}

""",
    "",
)

replace_once(
    PRODUCT,
    r"""        '\'' | '"' | '`' => Some(value),
""",
    r"""        '\'' | '"' => Some(value),
""",
)

replace_once(
    PRODUCT,
    """    #[test]
    fn code_mode_consumes_string_brackets_before_quote_and_paren_boundaries() {
        let plan = build_injection_plan(
            SensitiveText::new(r#"if (value[0] == "{") {"#.to_owned()),
            true,
            config(InjectionMode::Code, 256),
            capabilities(),
        )
        .expect("Code mode needs the keyboard code capabilities");
        let InjectionPlan::Code(plan) = plan else {
            panic!("Code mode must produce a Code plan");
        };

        let first_quote = plan
            .actions()
            .iter()
            .position(|action| *action == super::CodeAction::Atom(crate::TextAtom::Scalar('"')))
            .expect("string opener");
        assert_eq!(
            &plan.actions()[first_quote..first_quote.saturating_add(7)],
            &[
                super::CodeAction::Atom(crate::TextAtom::Scalar('"')),
                super::CodeAction::Atom(crate::TextAtom::Scalar('{')),
                super::CodeAction::CursorRight,
                super::CodeAction::CursorRight,
                super::CodeAction::CursorRight,
                super::CodeAction::Atom(crate::TextAtom::Scalar(' ')),
                super::CodeAction::Atom(crate::TextAtom::Scalar('{')),
            ]
        );
    }
""",
    """    #[test]
    fn code_mode_keeps_string_brackets_literal_before_quote_and_paren_boundaries() {
        let plan = build_injection_plan(
            SensitiveText::new(r#"if (value[0] == "{") {"#.to_owned()),
            true,
            config(InjectionMode::Code, 256),
            capabilities(),
        )
        .expect("Code mode needs the keyboard code capabilities");
        let InjectionPlan::Code(plan) = plan else {
            panic!("Code mode must produce a Code plan");
        };

        let first_quote = plan
            .actions()
            .iter()
            .position(|action| *action == super::CodeAction::Atom(crate::TextAtom::Scalar('"')))
            .expect("string opener");
        assert_eq!(
            &plan.actions()[first_quote..first_quote.saturating_add(6)],
            &[
                super::CodeAction::Atom(crate::TextAtom::Scalar('"')),
                super::CodeAction::Atom(crate::TextAtom::Scalar('{')),
                super::CodeAction::CursorRight,
                super::CodeAction::CursorRight,
                super::CodeAction::Atom(crate::TextAtom::Scalar(' ')),
                super::CodeAction::Atom(crate::TextAtom::Scalar('{')),
            ]
        );
    }
""",
)

replace_once(
    PRODUCT,
    """    fn code_mode_uses_keyboard_code_actions_for_indentation_and_pairs() {
"""
    + """        let plan = build_injection_plan(
""",
    """    fn code_mode_uses_keyboard_code_actions_for_indentation_and_pairs() {
"""
    + """        let plan = build_injection_plan(
""",
)

# The only numeric assertion changed here is the same-line CursorRight count:
# the brace inside the quoted character literal is now intentionally literal.
marker = "fn code_mode_uses_keyboard_code_actions_for_indentation_and_pairs()"
path = ROOT / PRODUCT
text = path.read_text(encoding="utf-8")
start = text.index(marker)
end = text.index("    #[test]", start)
section = text[start:end]
if section.count("            5\n") != 1:
    raise SystemExit("unexpected pair-count assertion in code-mode unit test")
text = text[:start] + section.replace("            5\n", "            4\n") + text[end:]
path.write_text(text, encoding="utf-8")

marker = "fn code_mode_keeps_comment_and_string_delimiters_literal()"
text = path.read_text(encoding="utf-8")
start = text.index(marker)
end = text.index("    #[test]", start)
section = text[start:end]
if section.count("            3\n") != 1:
    raise SystemExit("unexpected comment/string CursorRight assertion")
text = text[:start] + section.replace("            3\n", "            1\n") + text[end:]
path.write_text(text, encoding="utf-8")

replace_once(
    MACOS,
    """        if expected.render_host_limited || observed.render_host_limited {
            return if expected.render_host_limited
                && observed.render_host_limited
                && expected.window_hash.is_some()
            {
                TargetComparison::Same
            } else {
                TargetComparison::UnavailableOrAmbiguous
            };
        }
""",
    """        if expected.render_host_limited {
            // The initial render-host classification selects a process/window
            // comparison policy for the whole session. Monaco may rebuild a
            // focused AX node that temporarily lacks the classification, but a
            // real process/window change was already rejected above.
            return if expected.window_hash.is_some() {
                TargetComparison::Same
            } else {
                TargetComparison::UnavailableOrAmbiguous
            };
        }
        if observed.render_host_limited {
            // A native-control session must not weaken its exact-focus promise
            // merely because a later node looks like a render host.
            return TargetComparison::UnavailableOrAmbiguous;
        }
""",
)

replace_once(
    MACOS,
    """    #[test]
    fn render_host_uses_stable_window_identity_when_focus_node_is_rebuilt() {
        let expected = target(42, Some(5), Some(7), true);
        let rebuilt_focus = target(42, Some(5), Some(8), true);
        let other_window = target(42, Some(6), Some(8), true);

        assert_eq!(
            MacTarget.compare(&expected, &rebuilt_focus),
            TargetComparison::Same
        );
        assert_eq!(
            MacTarget.compare(&expected, &other_window),
            TargetComparison::Changed
        );
    }
""",
    """    #[test]
    fn render_host_uses_stable_window_identity_when_focus_node_is_rebuilt() {
        let expected = target(42, Some(5), Some(7), true);
        let rebuilt_focus = target(42, Some(5), Some(8), true);
        let transiently_reclassified = target(42, Some(5), Some(9), false);
        let other_window = target(42, Some(6), Some(8), false);
        let other_process = target(43, Some(5), Some(8), false);

        assert_eq!(
            MacTarget.compare(&expected, &rebuilt_focus),
            TargetComparison::Same
        );
        assert_eq!(
            MacTarget.compare(&expected, &transiently_reclassified),
            TargetComparison::Same
        );
        assert_eq!(
            MacTarget.compare(&expected, &other_window),
            TargetComparison::Changed
        );
        assert_eq!(
            MacTarget.compare(&expected, &other_process),
            TargetComparison::Changed
        );
    }
""",
)

replace_once(
    "apps/cliptype-flutter/pubspec.yaml",
    "version: 0.2.0-beta.2+1\n",
    "version: 0.2.0-beta.3+2\n",
)
