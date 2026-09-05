#!/usr/bin/env python3
"""Prove that the mock-Quartz gate rejects regressions, not just good input.

Mutations exist only in temporary copies of the public source fixture. These
checks do not read the system clipboard, query a real target, or post input.
"""
from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
VERIFY = ROOT / "scripts/verify_code_navigation.py"
SOURCE = ROOT / "crates/cliptype-macos/native/cliptype_macos.m"


class CodeNavigationGateTests(unittest.TestCase):
    def run_gate(
        self, source: Path, *, environment: dict[str, str] | None = None
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(VERIFY), "--source", str(source)],
            cwd=ROOT,
            env=environment,
            text=True,
            capture_output=True,
            timeout=60,
            check=False,
        )

    def test_production_source_passes_with_sanitizers(self) -> None:
        result = self.run_gate(SOURCE)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("native_contract_tests=15 failures=0", result.stdout)

    def assert_mutation_rejected(self, before: str, after: str) -> None:
        text = SOURCE.read_text(encoding="utf-8")
        self.assertIn(before, text, "mutation boundary changed; review this test")
        with tempfile.TemporaryDirectory(prefix="cliptype-gate-mutation-") as temp:
            source = Path(temp) / "mutated.m"
            source.write_text(text.replace(before, after), encoding="utf-8")
            result = self.run_gate(source)
        self.assertNotEqual(result.returncode, 0, "unsafe mutation passed the gate")
        # A missing compiler or syntax error is not proof of a caught regression.
        self.assertIn("native_contract_tests=15", result.stdout)
        self.assertIn("FAIL ", result.stderr)

    def test_combined_synthetic_event_source_is_rejected(self) -> None:
        self.assert_mutation_rejected(
            "CGEventSourceCreate(kCGEventSourceStatePrivate)",
            "CGEventSourceCreate(kCGEventSourceStateCombinedSessionState)",
        )

    def test_combined_modifier_observation_is_rejected(self) -> None:
        self.assert_mutation_rejected(
            "CGEventSourceFlagsState(kCGEventSourceStateHIDSystemState)",
            "CGEventSourceFlagsState(kCGEventSourceStateCombinedSessionState)",
        )

    def test_lingering_command_on_navigation_key_up_is_rejected(self) -> None:
        self.assert_mutation_rejected(
            "const CGEventFlags flags[4] = {0, 0, kCGEventFlagMaskCommand, 0};",
            "const CGEventFlags flags[4] = "
            "{0, 0, kCGEventFlagMaskCommand, kCGEventFlagMaskCommand};",
        )

    def test_missing_compiler_fails_instead_of_skipping(self) -> None:
        environment = dict(os.environ, CC="cliptype-deliberately-missing-compiler")
        result = self.run_gate(SOURCE, environment=environment)
        self.assertEqual(result.returncode, 2)
        self.assertIn("A C compiler is required", result.stderr)
        self.assertNotIn("failures=0", result.stdout)

    def test_missing_source_fails_instead_of_skipping(self) -> None:
        with tempfile.TemporaryDirectory(prefix="cliptype-gate-missing-") as temp:
            result = self.run_gate(Path(temp) / "missing.m")
        self.assertEqual(result.returncode, 2)
        self.assertIn("Source file not found", result.stderr)
        self.assertNotIn("failures=0", result.stdout)


if __name__ == "__main__":
    unittest.main()
