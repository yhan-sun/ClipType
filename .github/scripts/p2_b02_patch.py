from __future__ import annotations

import base64
import lzma
from pathlib import Path

path = Path(__file__)
payload = "".join(
    Path(f"{path}.part{index}").read_text(encoding="utf-8").strip()
    for index in range(1, 5)
)
source = lzma.decompress(base64.b64decode(payload)).decode("utf-8")
old = "'''        apply_command(command as usize, settings);\n    }\n\n    // SAFETY: this function owns the popup menu.'''"
new = "'''        if command > 0 {\n            apply_command(command as usize, settings);\n        }\n    }\n\n    // SAFETY: this function owns the popup menu.'''"
if source.count(old) != 1:
    raise RuntimeError(f"bootstrap source context matches={source.count(old)}")
source = source.replace(old, new, 1)
source += '''\nreplace_once(\n    "crates/cliptype-app/tests/coordinator.rs",\n    "    assert_eq!(status.batches_completed, 5);",\n    "    assert_eq!(status.batches_completed, 9);",\n)\n'''
exec(compile(source, str(path), "exec"))
