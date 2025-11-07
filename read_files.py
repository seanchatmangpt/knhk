#!/usr/bin/env python3
import sys
import os

files_to_check = [
    'rust/knhk-aot/src/lib.rs',
    'rust/knhk-aot/src/template_analyzer.rs',
    'rust/knhk-sidecar/src/service.rs',
    'rust/knhk-sidecar/Cargo.toml',
    'rust/knhk-warm/src/lib.rs',
    'rust/knhk-warm/Cargo.toml',
]

for filepath in files_to_check:
    full_path = os.path.join('/Users/sac/knhk', filepath)
    if os.path.exists(full_path):
        print(f"\n=== {filepath} ===")
        with open(full_path, 'r') as f:
            content = f.read()
            # Show first 100 lines
            lines = content.split('\n')
            print('\n'.join(lines[:100]))
    else:
        print(f"\n=== {filepath} NOT FOUND ===")

