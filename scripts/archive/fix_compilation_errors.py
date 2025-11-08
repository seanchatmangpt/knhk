#!/usr/bin/env python3
"""
Fix compilation errors across all Rust crates following core team standards.
"""
import os
import re
import sys

BASE_DIR = '/Users/sac/knhk'

def read_file(filepath):
    """Read file content."""
    full_path = os.path.join(BASE_DIR, filepath)
    if os.path.exists(full_path):
        with open(full_path, 'r') as f:
            return f.read()
    return None

def write_file(filepath, content):
    """Write file content."""
    full_path = os.path.join(BASE_DIR, filepath)
    os.makedirs(os.path.dirname(full_path), exist_ok=True)
    with open(full_path, 'w') as f:
        f.write(content)
    print(f"Fixed: {filepath}")

def fix_aot_no_std():
    """Fix knhk-aot no_std placement."""
    lib_rs = read_file('rust/knhk-aot/src/lib.rs')
    template_rs = read_file('rust/knhk-aot/src/template_analyzer.rs')
    
    if not lib_rs or not template_rs:
        print("Could not read knhk-aot files")
        return False
    
    # Check if no_std is in template_analyzer.rs
    if '#![no_std]' in template_rs:
        # Remove from template_analyzer.rs
        new_template = template_rs.replace('#![no_std]\n', '')
        write_file('rust/knhk-aot/src/template_analyzer.rs', new_template)
        
        # Add to lib.rs if not present
        if '#![no_std]' not in lib_rs:
            # Add after any existing attributes at the top
            lines = lib_rs.split('\n')
            insert_idx = 0
            for i, line in enumerate(lines):
                if line.startswith('#!'):
                    insert_idx = i + 1
                elif line.strip() and not line.startswith('//'):
                    break
            
            lines.insert(insert_idx, '#![no_std]')
            new_lib = '\n'.join(lines)
            write_file('rust/knhk-aot/src/lib.rs', new_lib)
            return True
    
    return False

def fix_sidecar_imports():
    """Check knhk-sidecar for import issues."""
    service_rs = read_file('rust/knhk-sidecar/src/service.rs')
    cargo_toml = read_file('rust/knhk-sidecar/Cargo.toml')
    
    if not service_rs or not cargo_toml:
        print("Could not read knhk-sidecar files")
        return False
    
    # Check if knhk-etl is in dependencies
    if 'knhk-etl' not in cargo_toml:
        # Add dependency
        deps_section = re.search(r'\[dependencies\](.*?)(?=\[|$)', cargo_toml, re.DOTALL)
        if deps_section:
            new_dep = 'knhk-etl = { path = "../knhk-etl", features = ["std"] }'
            if new_dep not in cargo_toml:
                # Insert after last dependency
                lines = cargo_toml.split('\n')
                for i, line in enumerate(lines):
                    if line.strip().startswith('knhk-'):
                        # Find the last knhk- dependency
                        last_idx = i
                        for j in range(i+1, len(lines)):
                            if lines[j].strip() and not lines[j].strip().startswith('['):
                                last_idx = j
                            else:
                                break
                        lines.insert(last_idx + 1, new_dep)
                        write_file('rust/knhk-sidecar/Cargo.toml', '\n'.join(lines))
                        return True
    
    return False

def main():
    """Main fix function."""
    print("=== Fixing compilation errors ===")
    
    fixes = []
    
    # Fix 1: knhk-aot no_std
    if fix_aot_no_std():
        fixes.append("knhk-aot: Moved #![no_std] to lib.rs root")
    
    # Fix 2: knhk-sidecar imports
    if fix_sidecar_imports():
        fixes.append("knhk-sidecar: Added knhk-etl dependency")
    
    print(f"\n=== Fixed {len(fixes)} issues ===")
    for fix in fixes:
        print(f"  - {fix}")

if __name__ == '__main__':
    main()

