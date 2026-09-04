#!/usr/bin/env python3
from pathlib import Path

path = Path("crates/cliptype-core/tests/code_mode_editor_regressions.rs")
text = path.read_text(encoding="utf-8")
replacements = [
    (
        "*generated && matches!(value, ')' | ']' | '}')",
        "*generated && matches!(*value, ')' | ']' | '}')",
    ),
    (
        ".take_while(|value| matches!(value, ' ' | '\\t'))",
        ".take_while(|value| matches!(**value, ' ' | '\\t'))",
    ),
]
for old, new in replacements:
    if text.count(old) != 1:
        raise SystemExit(f"expected one occurrence of {old!r}")
    text = text.replace(old, new)
path.write_text(text, encoding="utf-8")
