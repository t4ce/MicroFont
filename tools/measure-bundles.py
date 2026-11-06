#!/usr/bin/env python3
"""Measure the exact packed font payload for each feature set; Python stdlib only.

Run from any directory: python3 tools/measure-bundles.py
RUSTC can select a compiler command, e.g. RUSTC='rustc +1.95.0'.
This measures embedded font data, not linker-dependent executable code size.
"""
import json
import os
from pathlib import Path
import shlex
import subprocess
import tempfile

root = Path(__file__).resolve().parents[1]
with tempfile.TemporaryDirectory(prefix='microfont-sizes-') as temp:
    temp = Path(temp)
    packer = temp / 'packer'
    subprocess.run(shlex.split(os.environ.get('RUSTC', 'rustc')) + [
        '--edition=2024', str(root / 'build.rs'), '-o', str(packer)], check=True)
    sizes = {}
    for features in ['hex', 'raster', 'hex,raster']:
        out = temp / features.replace(',', '-')
        out.mkdir()
        env = {key: value for key, value in os.environ.items()
               if not key.startswith('CARGO_FEATURE_') and key != 'MICROFONT_SOURCE_DIR'}
        env.update(CARGO_MANIFEST_DIR=str(root), OUT_DIR=str(out), CARGO_CFG_TARGET_OS='linux')
        for feature in features.split(','):
            env['CARGO_FEATURE_' + feature.upper()] = '1'
        subprocess.run([str(packer)], env=env, check=True, stdout=subprocess.DEVNULL)
        sizes[features] = (out / 'microfont.bin').stat().st_size
    print(json.dumps({'unit': 'bytes', 'embedded_font_bundle': sizes}, indent=2))
