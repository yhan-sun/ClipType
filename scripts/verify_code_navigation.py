#!/usr/bin/env python3
"""Compile actual native input functions against a mock Quartz API.

This proves construction, isolation choices, and allocation cleanup. It does
NOT prove real Quartz state/timing, Accessibility, or editor compatibility.
No clipboard or target contents are read or printed.
"""
from __future__ import annotations
import argparse
from pathlib import Path
import os
import shlex
import shutil
import subprocess
import tempfile


def extract_native(text: str) -> str:
    start = text.index('uint64_t ct_macos_modifier_flags(void) {')
    end_start = text.index('int ct_macos_post_paste(int64_t expected_revision) {', start)
    # This fixed helper contains no nested blocks; reject unreviewed structure.
    end = text.index('\n}', end_start) + 2
    return text[start:end] + '\n'


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path)
    parser.add_argument('--no-sanitizers', action='store_true')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    source = args.source or root / 'crates/cliptype-macos/native/cliptype_macos.m'
    compiler = shlex.split(os.environ.get('CC', 'clang'))
    if not compiler or shutil.which(compiler[0]) is None:
        parser.error('A C compiler is required; install Clang or set CC.')
    if not source.is_file():
        parser.error(f'Source file not found: {source}')
    text = extract_native(source.read_text(encoding='utf-8'))
    with tempfile.TemporaryDirectory(prefix='cliptype-contract-') as temp:
        directory = Path(temp)
        (directory / 'native_keyboard_under_test.inc').write_text(text, encoding='utf-8')
        executable = directory / 'native-event-contract'
        command = compiler + ['-std=c11', '-Wall', '-Wextra', '-Werror', '-pedantic']
        if not args.no_sanitizers:
            command += ['-fsanitize=address,undefined', '-fno-omit-frame-pointer']
        command += ['-I', str(directory), str(root / 'scripts/tests/native_event_contract.c'), '-o', str(executable)]
        subprocess.run(command, check=True)
        return subprocess.run([str(executable)], check=False).returncode


if __name__ == '__main__':
    raise SystemExit(main())
