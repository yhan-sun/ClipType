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
source += '''\nreplace_once(\n    "crates/cliptype-app/tests/coordinator.rs",\n    "    assert_eq!(status.batches_completed, 5);",\n    "    assert_eq!(status.batches_completed, 9);",\n)\nreplace_once(\n    "crates/cliptype-app/tests/coordinator_edges.rs",\n    "        [Ok(1), Ok(1), Ok(2)],",\n    "        [Ok(1), Ok(1), Ok(1), Ok(2)],",\n)\nreplace_once(\n    "crates/cliptype-app/tests/coordinator_edges.rs",\n    """        [\n            ModifierObservation::Clear,\n            ModifierObservation::Clear,\n            ModifierObservation::Held(ModifierMask::SHIFT),\n        ],""",\n    """        [\n            ModifierObservation::Clear,\n            ModifierObservation::Clear,\n            ModifierObservation::Clear,\n            ModifierObservation::Held(ModifierMask::SHIFT),\n        ],""",\n)\nreplace_once(\n    "apps/cliptype/examples/p2_controlled_e2e.rs",\n    """                auto_clipboard_threshold: AutoClipboardThreshold::new(case.threshold)\n                    .map_err(|_| E2eError::CoordinatorConfiguration)?,\n                safety: P1Config {""",\n    """                auto_clipboard_threshold: AutoClipboardThreshold::new(case.threshold)\n                    .map_err(|_| E2eError::CoordinatorConfiguration)?,\n                jitter_percent: 0,\n                typo_probability_percent: 0,\n                safety: P1Config {""",\n)\nreplace_once(\n    "apps/cliptype/examples/p2_backend_benchmark.rs",\n    """                    auto_clipboard_threshold: AutoClipboardThreshold::new(256)\n                        .map_err(|_| BenchmarkError::CoordinatorConfiguration)?,\n                    safety: P1Config {""",\n    """                    auto_clipboard_threshold: AutoClipboardThreshold::new(256)\n                        .map_err(|_| BenchmarkError::CoordinatorConfiguration)?,\n                    jitter_percent: 0,\n                    typo_probability_percent: 0,\n                    safety: P1Config {""",\n)\n'''
exec(compile(source, str(path), "exec"))
