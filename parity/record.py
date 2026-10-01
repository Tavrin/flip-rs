#!/usr/bin/env python3
"""Append source and artifact identities without modifying either Git checkout."""
import hashlib
import os
from pathlib import Path
import subprocess
import sys


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


root = Path(__file__).resolve().parent.parent
revision = subprocess.check_output(['git', '--no-optional-locks', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
target = Path(os.environ.get('CARGO_TARGET_DIR', root / 'target'))
report = Path(sys.argv[1])
with report.open('a') as out:
    out.write('\n## Qualification binding\n\n')
    out.write(f'Rust base revision: `{revision}`; the measured working sources include the uncommitted hardening changes. The SHA-256 snapshot below binds those exact sources.\n\n')
    for path in sorted([root / 'Cargo.toml', root / 'Cargo.lock', *root.glob('src/*.rs'), *root.glob('examples/**/*.rs'), root / 'parity/reference.cpp', *root.glob('parity/*.sh')]):
        out.write(f'- `{path.relative_to(root)}`: `{sha(path)}`\n')
    for label, path in [('Rust parity/benchmark executable', target / 'release/examples/parity'), ('C++ oracle executable', root / 'parity/build/reference')]:
        out.write(f'- {label} SHA-256: `{sha(path)}`\n')
    for tool in [['rustc', '--version'], ['cargo', '--version'], ['g++', '-dumpfullversion']]:
        out.write(f'- {tool[0]}: `{subprocess.check_output(tool, text=True).strip()}`\n')
