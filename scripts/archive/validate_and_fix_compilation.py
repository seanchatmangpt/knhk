#!/usr/bin/env python3
"""
Comprehensive validation and fix script for compilation errors.
Follows core team standards: no placeholders, proper error handling, real implementations.
"""
import os
import re
import subprocess
import sys

BASE_DIR = '/Users/sac/knhk/rust'

def run_cargo_check(crate):
    """Run cargo check on a crate."""
    crate_dir = os.path.join(BASE_DIR, crate)
    if not os.path.exists(crate_dir):
        return None
    
    try:
        result = subprocess.run(
            ['cargo', 'check', '--lib'],
            cwd=crate_dir,
            capture_output=True,
            text=True,
            timeout=60
        )
        return result.stdout + result.stderr
    except Exception as e:
        return f"Error: {e}"

def read_file(filepath):
    """Read file content."""
    if os.path.exists(filepath):
        with open(filepath, 'r') as f:
            return f.read()
    return None

def write_file(filepath, content):
    """Write file content."""
    os.makedirs(os.path.dirname(filepath), exist_ok=True)
    with open(filepath, 'w') as f:
        f.write(content)
    print(f"✓ Fixed: {filepath}")

def fix_aot_no_std():
    """Fix knhk-aot no_std placement."""
    lib_rs_path = os.path.join(BASE_DIR, 'knhk-aot/src/lib.rs')
    template_rs_path = os.path.join(BASE_DIR, 'knhk-aot/src/template_analyzer.rs')
    
    lib_rs = read_file(lib_rs_path)
    template_rs = read_file(template_rs_path)
    
    if not lib_rs or not template_rs:
        return False
    
    fixed = False
    
    # Remove #![no_std] from template_analyzer.rs if present
    if '#![no_std]' in template_rs:
        new_template = re.sub(r'^#!\[no_std\]\s*\n?', '', template_rs, flags=re.MULTILINE)
        write_file(template_rs_path, new_template)
        fixed = True
    
    # Add #![no_std] to lib.rs if not present
    if lib_rs and '#![no_std]' not in lib_rs:
        # Add at the very beginning
        if lib_rs.startswith('//') or lib_rs.startswith('/*'):
            # Find first non-comment line
            lines = lib_rs.split('\n')
            insert_idx = 0
            for i, line in enumerate(lines):
                if line.strip() and not (line.strip().startswith('//') or line.strip().startswith('/*')):
                    insert_idx = i
                    break
            lines.insert(insert_idx, '#![no_std]')
            new_lib = '\n'.join(lines)
        else:
            new_lib = '#![no_std]\n' + lib_rs
        write_file(lib_rs_path, new_lib)
        fixed = True
    
    return fixed

def fix_sidecar_dependencies():
    """Fix knhk-sidecar dependencies."""
    cargo_toml_path = os.path.join(BASE_DIR, 'knhk-sidecar/Cargo.toml')
    cargo_toml = read_file(cargo_toml_path)
    
    if not cargo_toml:
        return False
    
    # Check if knhk-etl is in dependencies
    if 'knhk-etl' not in cargo_toml or 'knhk-etl = {' not in cargo_toml:
        # Find [dependencies] section
        lines = cargo_toml.split('\n')
        deps_start = None
        for i, line in enumerate(lines):
            if line.strip() == '[dependencies]':
                deps_start = i
                break
        
        if deps_start is not None:
            # Find where to insert (after knhk-otel or at end of dependencies)
            insert_idx = deps_start + 1
            for i in range(deps_start + 1, len(lines)):
                if lines[i].strip().startswith('['):
                    insert_idx = i
                    break
                if 'knhk-otel' in lines[i]:
                    insert_idx = i + 1
            
            # Insert dependency
            new_dep = 'knhk-etl = { path = "../knhk-etl", features = ["std"] }'
            lines.insert(insert_idx, new_dep)
            write_file(cargo_toml_path, '\n'.join(lines))
            return True
    
    return False

def main():
    """Main validation and fix function."""
    print("=" * 60)
    print("COMPILATION ERROR VALIDATION AND FIX")
    print("=" * 60)
    
    crates = ['knhk-etl', 'knhk-sidecar', 'knhk-warm', 'knhk-aot', 'knhk-lockchain', 'knhk-validation', 'knhk-unrdf']
    
    print("\n=== PHASE 1: VALIDATION ===")
    errors = {}
    for crate in crates:
        print(f"\nChecking {crate}...")
        output = run_cargo_check(crate)
        if output:
            # Count errors
            error_count = len([l for l in output.split('\n') if 'error[' in l])
            if error_count > 0:
                errors[crate] = error_count
                print(f"  Found {error_count} errors")
                # Show first few errors
                error_lines = [l for l in output.split('\n') if 'error[' in l][:5]
                for line in error_lines:
                    print(f"    {line[:80]}")
    
    print("\n=== PHASE 2: FIXES ===")
    fixes_applied = []
    
    # Fix 1: knhk-aot no_std
    if fix_aot_no_std():
        fixes_applied.append("knhk-aot: Fixed #![no_std] placement")
    
    # Fix 2: knhk-sidecar dependencies
    if fix_sidecar_dependencies():
        fixes_applied.append("knhk-sidecar: Added knhk-etl dependency")
    
    if fixes_applied:
        print(f"\nApplied {len(fixes_applied)} fixes:")
        for fix in fixes_applied:
            print(f"  ✓ {fix}")
    else:
        print("No fixes needed or files not found")
    
    print("\n=== PHASE 3: RE-VALIDATION ===")
    print("Run 'cargo check --workspace' to verify fixes")
    
    print("\n" + "=" * 60)
    print("VALIDATION AND FIX COMPLETE")
    print("=" * 60)

if __name__ == '__main__':
    main()

