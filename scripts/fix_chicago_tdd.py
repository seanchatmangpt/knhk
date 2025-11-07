#!/usr/bin/env python3
"""
Fix Chicago TDD test compilation errors
Validates and fixes common issues in Chicago TDD test files
"""

import os
import re
import sys
from pathlib import Path

def find_chicago_tdd_tests(root_dir):
    """Find all Chicago TDD test files"""
    test_files = []
    for path in Path(root_dir).rglob("*chicago_tdd*.rs"):
        if path.is_file():
            test_files.append(path)
    return sorted(test_files)

def fix_unwrap_calls(content):
    """Replace unwrap() with proper error handling in tests"""
    # In tests, expect() is acceptable, but unwrap() should be replaced
    # Pattern: .unwrap() -> .expect("descriptive message")
    lines = content.split('\n')
    fixed_lines = []
    for line in lines:
        # Replace unwrap() with expect() in test contexts
        if 'unwrap()' in line and ('#[test]' in '\n'.join(fixed_lines[-5:]) or 'fn test_' in '\n'.join(fixed_lines[-5:])):
            # Try to create a descriptive message
            if 'result' in line.lower():
                line = line.replace('.unwrap()', '.expect("Test should succeed")')
            elif 'value' in line.lower() or 'val' in line.lower():
                line = line.replace('.unwrap()', '.expect("Test value should exist")')
            else:
                line = line.replace('.unwrap()', '.expect("Test assertion failed")')
        fixed_lines.append(line)
    return '\n'.join(fixed_lines)

def fix_uppercase_variables(content):
    """Fix uppercase S, P, O variables to lowercase"""
    # Pattern: let S = -> let s =
    # Pattern: let P = -> let p =
    # Pattern: let O = -> let o =
    content = re.sub(r'\blet\s+S\s*=', 'let s =', content)
    content = re.sub(r'\blet\s+P\s*=', 'let p =', content)
    content = re.sub(r'\blet\s+O\s*=', 'let o =', content)
    # Also fix in tuple destructuring: let (S, P, O) = -> let (s, p, o) =
    content = re.sub(r'\blet\s+\(S,\s*P,\s*O\)\s*=', 'let (s, p, o) =', content)
    return content

def fix_missing_debug_trait(content):
    """Add Debug trait to structs used in tests"""
    # Pattern: pub struct X { -> #[derive(Debug)]\npub struct X {
    # Only if Debug is not already present
    if '#[derive(Debug)]' not in content and 'pub struct' in content:
        # This is complex - would need to parse properly
        # For now, just note it
        pass
    return content

def fix_test_file(file_path):
    """Fix a single test file"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Apply fixes
        content = fix_uppercase_variables(content)
        content = fix_unwrap_calls(content)
        content = fix_missing_debug_trait(content)
        
        # Only write if changed
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✅ Fixed: {file_path}")
            return True
        else:
            print(f"✓ No changes needed: {file_path}")
            return False
    except Exception as e:
        print(f"❌ Error fixing {file_path}: {e}")
        return False

def main():
    root_dir = Path(__file__).parent.parent
    rust_dir = root_dir / "rust"
    
    if not rust_dir.exists():
        print(f"Error: {rust_dir} does not exist")
        sys.exit(1)
    
    print("=== Chicago TDD Test Fix Script ===")
    print(f"Searching in: {rust_dir}")
    print()
    
    test_files = find_chicago_tdd_tests(rust_dir)
    
    if not test_files:
        print("No Chicago TDD test files found")
        sys.exit(0)
    
    print(f"Found {len(test_files)} Chicago TDD test files:")
    for f in test_files:
        print(f"  - {f.relative_to(root_dir)}")
    print()
    
    print("Fixing test files...")
    fixed_count = 0
    for test_file in test_files:
        if fix_test_file(test_file):
            fixed_count += 1
    
    print()
    print(f"=== Fix Complete ===")
    print(f"Fixed {fixed_count} out of {len(test_files)} files")

if __name__ == "__main__":
    main()

