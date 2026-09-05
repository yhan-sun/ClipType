#!/usr/bin/env python3
import base64
import json
import pathlib
import zlib

root = pathlib.Path(__file__).resolve().parents[2]
parts = [
    (root / ".github/code-fix/data.00").read_text().strip(),
    (root / ".github/code-fix/data.01").read_text().strip(),
    (root / ".github/code-fix/data.02").read_text().strip(),
]
data = json.loads(zlib.decompress(base64.b64decode("".join(parts))))

for item in data["r"]:
    path = root / item["path"]
    text = path.read_text()
    before = item["before"]
    after = item["after"]
    count = text.count(before)
    if count != 1:
        raise SystemExit(
            f"{item['path']}: expected one replacement target, found {count}"
        )
    path.write_text(text.replace(before, after, 1))

for rel, content in data["n"].items():
    path = root / rel
    if path.exists():
        raise SystemExit(f"{rel}: destination already exists")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)

print(
    f"applied {len(data['r'])} replacements and created {len(data['n'])} files"
)
