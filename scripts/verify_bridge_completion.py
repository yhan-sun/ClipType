#!/usr/bin/env python3
"""Compile the production Swift completion mapping and verify the C ABI.

Uses the real RustSnapshot source and bridge header. This does not link the
Rust runtime, build Flutter, or claim an interactive application test.
"""
from __future__ import annotations

import argparse
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile

# The contract is explicit; old values 0..6 must not be renumbered.
EXPECTED = [None, 'completed', 'cancelled', 'target_changed',
            'clipboard_changed', 'permission', 'failed', 'modifier_conflict',
            'target_evidence_unavailable', 'target_disappeared', 'partial_input',
            'progress_unknown', 'blocked_cause_unknown', 'native_failure',
            'internal_invariant', 'modifier_timeout']


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--swift', type=Path)
    parser.add_argument('--header', type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    runner = root / 'apps/cliptype-flutter/macos/Runner'
    swift_path = args.swift or runner / 'RustBridge.swift'
    header = (args.header or runner / 'cliptype_bridge.h').resolve()
    cc = shlex.split(os.environ.get('CC', 'clang'))
    swiftc = shlex.split(os.environ.get('SWIFTC', 'swiftc'))
    for command in (cc, swiftc):
        if not command or shutil.which(command[0]) is None:
            parser.error('Clang and Swift are required; set CC/SWIFTC as needed.')
    if not swift_path.is_file() or not header.is_file():
        parser.error('Both the production Swift source and bridge header are required.')
    text = swift_path.read_text(encoding='utf-8')
    if text.count('final class RustBridge {') != 1:
        parser.error('RustBridge source boundary changed; review the test extractor.')
    snapshot = text.split('final class RustBridge {')[0]
    expected = ', '.join('nil' if value is None else f'"{value}"' for value in EXPECTED)
    with tempfile.TemporaryDirectory(prefix='cliptype-bridge-contract-') as temp:
        directory = Path(temp)
        (directory / 'Snapshot.swift').write_text(snapshot, encoding='utf-8')
        (directory / 'main.swift').write_text('''import Foundation
let expected: [String?] = [''' + expected + ''']
func snapshot(_ completion: Int) -> RustSnapshot {
    RustSnapshot(enabled: true, notifications: true, startAtLogin: false,
        mode: 3, charactersPerSecond: 100, jitterPercent: 0,
        typoProbabilityPercent: 0, autoClipboardThreshold: 256,
        generation: 1, phase: 0, backend: 2, completion: completion,
        batchesCompleted: 287)
}
var failures = 0
for (code, name) in expected.enumerated() {
    if snapshot(code).completionName != name {
        print("FAIL completion mapping code=\\(code)")
        failures += 1
    }
}
if snapshot(999).completionName != nil { failures += 1 }
if snapshot(0).backendName != "code" { failures += 1 }
if snapshot(0).phaseName != "idle" { failures += 1 }
print("swift_mapping_tests=19 failures=\\(failures) scope=production_snapshot_only")
exit(failures == 0 ? 0 : 1)
''', encoding='utf-8')
        (directory / 'abi.c').write_text('''#include <stddef.h>
#include <stdio.h>
#include "cliptype_bridge.h"
_Static_assert(sizeof(void *) == 8, "this preview contract is 64-bit");
_Static_assert(sizeof(CTBridgeState) == 48, "state size changed");
_Static_assert(offsetof(CTBridgeState, generation) == 24, "generation moved");
_Static_assert(offsetof(CTBridgeState, completion) == 40, "completion moved");
_Static_assert(offsetof(CTBridgeState, batches_completed) == 44, "count moved");
_Static_assert(CT_BRIDGE_COMPLETION_NONE == 0, "existing code changed");
_Static_assert(CT_BRIDGE_COMPLETION_FAILED == 6, "existing code changed");
int main(void) {
    puts("c_abi_assertions=7 failures=0 scope=64bit_header_layout");
    return 0;
}
''', encoding='utf-8')
        subprocess.run(cc + ['-std=c11', '-Wall', '-Wextra', '-Werror', '-pedantic',
                            '-I', str(header.parent), str(directory / 'abi.c'),
                            '-o', str(directory / 'abi')], check=True)
        subprocess.run([str(directory / 'abi')], check=True)
        subprocess.run(swiftc + ['-warnings-as-errors', '-import-objc-header', str(header),
                                str(directory / 'Snapshot.swift'), str(directory / 'main.swift'),
                                '-o', str(directory / 'snapshot')], check=True)
        return subprocess.run([str(directory / 'snapshot')], check=False).returncode


if __name__ == '__main__':
    raise SystemExit(main())
