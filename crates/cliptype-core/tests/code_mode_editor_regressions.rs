use cliptype_core::{
    CapabilityState, CodeAction, InjectionBackend, InjectionMode, InjectionPlan, PlanCapabilities,
    ProductCapabilities, ProductConfig, SensitiveText, TextAtom, build_injection_plan,
};

fn actions_for(source: &str) -> Vec<CodeAction> {
    let capabilities = ProductCapabilities {
        keyboard: PlanCapabilities {
            unicode_text: CapabilityState::Available,
            line_break: CapabilityState::Available,
            tab: CapabilityState::Available,
            cursor_right: CapabilityState::Available,
            modifier_observation: CapabilityState::Available,
        },
        clipboard_paste: CapabilityState::Unavailable,
        clipboard_revision_guard: CapabilityState::Unavailable,
    };
    let config = ProductConfig {
        mode: InjectionMode::Code,
        ..ProductConfig::default()
    };
    let plan = build_injection_plan(
        SensitiveText::new(source.to_owned()),
        false,
        config,
        capabilities,
    )
    .expect("Code mode must plan with keyboard capabilities only");

    assert_eq!(plan.backend(), InjectionBackend::Code);
    let InjectionPlan::Code(plan) = plan else {
        panic!("Code mode must produce a Code plan");
    };
    plan.actions().to_vec()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorLexState {
    Normal,
    LineComment,
    BlockComment { previous_star: bool },
    String { delimiter: char, escaped: bool },
}

struct EditorModel {
    text: Vec<char>,
    generated: Vec<bool>,
    caret: usize,
    state: EditorLexState,
    pending_slash: bool,
}

impl Default for EditorModel {
    fn default() -> Self {
        Self {
            text: Vec::new(),
            generated: Vec::new(),
            caret: 0,
            state: EditorLexState::Normal,
            pending_slash: false,
        }
    }
}

impl EditorModel {
    fn render(actions: &[CodeAction]) -> String {
        let mut editor = Self::default();
        for action in actions {
            editor.apply(*action);
        }
        editor.text.into_iter().collect()
    }

    fn apply(&mut self, action: CodeAction) {
        match action {
            CodeAction::Atom(atom) => self.type_atom(atom),
            CodeAction::CursorRight => self.cursor_right(),
            CodeAction::CursorRightToLineEnd => self.cursor_right_to_line_end(),
        }
    }

    fn type_atom(&mut self, atom: TextAtom) {
        match atom {
            TextAtom::Scalar(value) => self.type_scalar(value),
            TextAtom::LineBreak => self.type_line_break(),
            TextAtom::Tab => self.insert_literal('\t'),
        }
    }

    fn type_scalar(&mut self, value: char) {
        match self.state {
            EditorLexState::Normal => self.type_normal_scalar(value),
            EditorLexState::LineComment => self.insert_literal(value),
            EditorLexState::BlockComment { previous_star } => {
                self.insert_literal(value);
                self.state = if previous_star && value == '/' {
                    EditorLexState::Normal
                } else {
                    EditorLexState::BlockComment {
                        previous_star: value == '*',
                    }
                };
            }
            EditorLexState::String { delimiter, escaped } => {
                self.insert_literal(value);
                self.state = if escaped {
                    EditorLexState::String {
                        delimiter,
                        escaped: false,
                    }
                } else if value == '\\' {
                    EditorLexState::String {
                        delimiter,
                        escaped: true,
                    }
                } else {
                    EditorLexState::String {
                        delimiter,
                        escaped: false,
                    }
                };
            }
        }
    }

    fn type_normal_scalar(&mut self, value: char) {
        if self.pending_slash {
            self.pending_slash = false;
            if value == '/' {
                self.insert_literal(value);
                self.state = EditorLexState::LineComment;
                return;
            }
            if value == '*' {
                self.insert_literal(value);
                self.state = EditorLexState::BlockComment {
                    previous_star: false,
                };
                return;
            }
        }

        if let Some(closer) = generated_pair(value) {
            self.insert_pair(value, closer);
            if matches!(value, '\'' | '"') {
                self.state = EditorLexState::String {
                    delimiter: value,
                    escaped: false,
                };
            }
            return;
        }

        self.insert_literal(value);
        self.pending_slash = value == '/';
    }

    fn type_line_break(&mut self) {
        let indent = self.current_line_indent();
        let closer_waiting = self
            .text
            .get(self.caret)
            .zip(self.generated.get(self.caret))
            .is_some_and(|(value, generated)| {
                *generated && matches!(value, ')' | ']' | '}')
            });

        let mut insertion = String::from("\n");
        if closer_waiting {
            insertion.push_str(&" ".repeat(indent + 4));
            insertion.push('\n');
        }
        insertion.push_str(&" ".repeat(indent));

        let original_caret = self.caret;
        self.insert_sequence(original_caret, insertion.chars());
        self.caret = original_caret + 1 + if closer_waiting { indent + 4 } else { indent };

        if self.state == EditorLexState::LineComment {
            self.state = EditorLexState::Normal;
        }
        self.pending_slash = false;
    }

    fn cursor_right(&mut self) {
        let skipped = *self
            .text
            .get(self.caret)
            .expect("CursorRight must have an editor-generated closer");
        assert!(
            self.generated[self.caret],
            "CursorRight must not skip source text: {skipped:?}"
        );
        self.caret += 1;

        if let EditorLexState::String { delimiter, .. } = self.state {
            assert_eq!(skipped, delimiter, "string navigation must skip its quote");
            self.state = EditorLexState::Normal;
        }
        self.pending_slash = false;
    }

    fn cursor_right_to_line_end(&mut self) {
        assert_eq!(
            self.text.get(self.caret),
            Some(&'\n'),
            "line-closer navigation must start at an existing line boundary"
        );
        self.caret += 1;
        let mut saw_generated_closer = false;
        while self.caret < self.text.len() && self.text[self.caret] != '\n' {
            saw_generated_closer |= self.generated[self.caret];
            self.caret += 1;
        }
        assert!(
            saw_generated_closer,
            "line-end navigation must cross an editor-generated closer"
        );

        if self.state == EditorLexState::LineComment {
            self.state = EditorLexState::Normal;
        }
        self.pending_slash = false;
    }

    fn current_line_indent(&self) -> usize {
        let start = self.text[..self.caret]
            .iter()
            .rposition(|value| *value == '\n')
            .map_or(0, |index| index + 1);
        self.text[start..self.caret]
            .iter()
            .take_while(|value| matches!(value, ' ' | '\t'))
            .map(|value| if *value == '\t' { 4 } else { 1 })
            .sum()
    }

    fn insert_pair(&mut self, opener: char, closer: char) {
        self.text.insert(self.caret, opener);
        self.generated.insert(self.caret, false);
        self.caret += 1;
        self.text.insert(self.caret, closer);
        self.generated.insert(self.caret, true);
    }

    fn insert_literal(&mut self, value: char) {
        self.text.insert(self.caret, value);
        self.generated.insert(self.caret, false);
        self.caret += 1;
    }

    fn insert_sequence(&mut self, index: usize, values: impl IntoIterator<Item = char>) {
        for (offset, value) in values.into_iter().enumerate() {
            self.text.insert(index + offset, value);
            self.generated.insert(index + offset, false);
        }
    }
}

const fn generated_pair(value: char) -> Option<char> {
    match value {
        '(' => Some(')'),
        '{' => Some('}'),
        '[' => Some(']'),
        '"' => Some('"'),
        '\'' => Some('\''),
        _ => None,
    }
}

#[test]
fn exact_rust_fixture_matches_vscode_like_auto_pair_and_indent_output() {
    let source = concat!(
        "fn main() {\n",
        "let value = {\"x\": [1, 2]};\n",
        "if (value[0] == \"{\") {\n",
        "println!(\"value = {}\", value);\n",
        "}\n",
        "}\n",
        "NEXT",
    );
    let expected = concat!(
        "fn main() {\n",
        "    let value = {\"x\": [1, 2]};\n",
        "    if (value[0] == \"{\") {\n",
        "        println!(\"value = {}\", value);\n",
        "    }\n",
        "}\n",
        "NEXT",
    );

    assert_eq!(EditorModel::render(&actions_for(source)), expected);
}

#[test]
fn cpp_multifunction_fixture_preserves_chinese_comments_arrays_and_string_brackets() {
    let source = concat!(
        "int sum(const int values[], int size) {\n",
        "int total = 0;\n",
        "for (int i = 0; i < size; ++i) {\n",
        "total += values[i];\n",
        "}\n",
        "return total;\n",
        "}\n",
        "\n",
        "int main() {\n",
        "int values[] = {1, 2, 3};\n",
        "const char* text = \"{[()]}\"; // 中文注释：字符串括号按字面输入\n",
        "if (sum(values, 3) == 6) {\n",
        "return 0;\n",
        "// 中文注释：下一行闭括号由编辑器生成\n",
        "}\n",
        "return 1;\n",
        "}\n",
        "NEXT",
    );
    let expected = concat!(
        "int sum(const int values[], int size) {\n",
        "    int total = 0;\n",
        "    for (int i = 0; i < size; ++i) {\n",
        "        total += values[i];\n",
        "    }\n",
        "    return total;\n",
        "}\n",
        "\n",
        "int main() {\n",
        "    int values[] = {1, 2, 3};\n",
        "    const char* text = \"{[()]}\"; // 中文注释：字符串括号按字面输入\n",
        "    if (sum(values, 3) == 6) {\n",
        "        return 0;\n",
        "        // 中文注释：下一行闭括号由编辑器生成\n",
        "    }\n",
        "    return 1;\n",
        "}\n",
        "NEXT",
    );

    assert_eq!(EditorModel::render(&actions_for(source)), expected);
}

#[test]
fn pair_navigation_is_limited_to_the_five_documented_pair_families() {
    let actions = actions_for("(){}[]\"\"''");
    assert_eq!(
        actions
            .iter()
            .filter(|action| matches!(action, CodeAction::CursorRight))
            .count(),
        5
    );

    let literal = "let marker = `code`; <value>";
    assert!(
        actions_for(literal)
            .iter()
            .all(|action| matches!(action, CodeAction::Atom(_)))
    );
}

#[test]
fn triple_quote_boundaries_and_bodies_never_use_pair_navigation() {
    for source in [
        "const doc = \"\"\"hello {[]}\"\"\";",
        "const doc = '''hello {[]}''';",
    ] {
        assert!(
            actions_for(source)
                .iter()
                .all(|action| matches!(action, CodeAction::Atom(_)))
        );
    }
}

#[test]
fn long_code_fixture_reaches_tail_after_every_generated_closing_line() {
    let mut source = String::new();
    let mut expected = String::new();
    for index in 0..64 {
        source.push_str(&format!(
            "int function_{index}(int value) {{\nif ((value + {index}) > 0) {{\nreturn value;\n// 中文注释 {index}\n}}\nreturn -1;\n}}\n"
        ));
        expected.push_str(&format!(
            "int function_{index}(int value) {{\n    if ((value + {index}) > 0) {{\n        return value;\n        // 中文注释 {index}\n    }}\n    return -1;\n}}\n"
        ));
    }
    source.push_str("NEXT");
    expected.push_str("NEXT");

    let actions = actions_for(&source);
    assert_eq!(
        actions
            .iter()
            .filter(|action| matches!(action, CodeAction::CursorRightToLineEnd))
            .count(),
        64 * 2
    );
    assert_eq!(
        actions.last(),
        Some(&CodeAction::Atom(TextAtom::Scalar('T')))
    );
    assert_eq!(EditorModel::render(&actions), expected);
}
