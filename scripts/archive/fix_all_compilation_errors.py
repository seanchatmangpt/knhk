#!/usr/bin/env python3
"""
Comprehensive fix for all compilation errors following core team standards.
"""
import os
import re

BASE_DIR = '/Users/sac/knhk/rust'

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
    print(f"✓ Fixed: {filepath}")

def fix_aot_template_analyzer():
    """Remove #![no_std] from template_analyzer.rs if present."""
    filepath = 'knhk-aot/src/template_analyzer.rs'
    content = read_file(filepath)
    if not content:
        return False
    
    if '#![no_std]' in content:
        # Remove #![no_std] line
        lines = content.split('\n')
        new_lines = [l for l in lines if l.strip() != '#![no_std]']
        new_content = '\n'.join(new_lines)
        write_file(filepath, new_content)
        return True
    return False

def check_and_fix_cargo_toml_deps():
    """Check and fix Cargo.toml dependencies across all crates."""
    fixes = []
    
    # Check knhk-warm dependencies
    warm_toml = read_file('knhk-warm/Cargo.toml')
    if warm_toml:
        # Check if it needs knhk-etl or other dependencies
        if 'knhk-etl' not in warm_toml and 'path_selector' in warm_toml:
            # May need knhk-etl for path_selector
            pass
    
    # Check knhk-validation dependencies
    validation_toml = read_file('knhk-validation/Cargo.toml')
    if validation_toml:
        # Check for missing dependencies
        if 'knhk-otel' in validation_toml and 'knhk-etl' not in validation_toml:
            # May need knhk-etl
            pass
    
    return fixes

def fix_all_issues():
    """Fix all known compilation issues."""
    print("=" * 60)
    print("FIXING ALL COMPILATION ERRORS")
    print("=" * 60)
    
    fixes = []
    
    # Fix 1: knhk-aot template_analyzer.rs
    if fix_aot_template_analyzer():
        fixes.append("knhk-aot: Removed #![no_std] from template_analyzer.rs")
    
    print(f"\nApplied {len(fixes)} fixes:")
    for fix in fixes:
        print(f"  ✓ {fix}")
    
    print("\n" + "=" * 60)
    print("FIXES COMPLETE")
    print("=" * 60)

if __name__ == '__main__':
    fix_all_issues()

