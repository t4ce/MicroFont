#!/usr/bin/env python3
"""Two-profile linked-code estimates for x86_64 Linux; Python stdlib + Cargo + GNU size.

Builds (never executes) no_std, no-allocator, statically linked ELF probes.
Reports .text deltas against the same empty baseline, excluding font payload and
other non-code sections. The generated entry point parks forever: do not run it.
"""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain', help='rustup toolchain, e.g. 1.95.0')
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
selector = ['+' + args.toolchain] if args.toolchain else []
compiler = subprocess.check_output(['rustc', *selector, '--version'], text=True).strip()
target = 'x86_64-unknown-linux-gnu'
with tempfile.TemporaryDirectory(prefix='microfont-code-') as temp:
    temp = Path(temp)
    (temp / 'src').mkdir()
    shutil.copy2(root / 'tools/code-size-probe.rs', temp / 'src/main.rs')
    manifest = '''[package]
name = "microfont-code-size-probe"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
microfont = { path = PATH, default-features = false }
[features]
baseline = ["microfont/hex"]
hex = ["microfont/stamp"]
svg = ["microfont/raster"]
[profile.release]
panic = "abort"
lto = "fat"
codegen-units = 1
strip = "symbols"
[profile.small]
inherits = "release"
opt-level = "z"
'''.replace('PATH', json.dumps(str(root)))
    (temp / 'Cargo.toml').write_text(manifest)
    env = {key: value for key, value in os.environ.items()
           if key not in ('CARGO_ENCODED_RUSTFLAGS', 'RUSTFLAGS', 'MICROFONT_SOURCE_DIR')}
    env['RUSTFLAGS'] = '-C link-arg=-nostartfiles -C link-arg=-static -C relocation-model=static'
    # Explicit target keeps freestanding link flags away from host build scripts.
    env['CARGO_TARGET_DIR'] = str(temp / 'target')
    profiles = {}
    for profile in ['release', 'small']:
        sizes = {}
        for feature in ['baseline', 'hex', 'svg']:
            subprocess.run(['cargo', *selector, 'build', '--manifest-path', str(temp / 'Cargo.toml'),
                            '--target', target, '--profile', profile, '--no-default-features',
                            '--features', feature, '--quiet'], cwd=temp, env=env, check=True)
            binary = temp / 'target' / target / profile / 'microfont-code-size-probe'
            sections = subprocess.check_output(['size', '-A', str(binary)], text=True)
            sizes[feature] = int(re.search(r'^\.text\s+(\d+)\b', sections, re.M)[1])
        profiles[profile] = dict(baseline_text=sizes['baseline'],
                                 hex_stamp_text=sizes['hex'] - sizes['baseline'],
                                 raster_svg_text=sizes['svg'] - sizes['baseline'])
    averages = {key: sum(row[key] for row in profiles.values()) / len(profiles)
                for key in ['hex_stamp_text', 'raster_svg_text']}
    print(json.dumps(dict(compiler=compiler, target=target, unit='incremental .text bytes',
                         profiles=profiles, two_profile_mean=averages), indent=2))
