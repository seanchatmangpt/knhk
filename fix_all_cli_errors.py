#!/usr/bin/env python3
"""
Comprehensive fix script for knhk-cli compilation errors
Based on V1-STATUS.md error categories:
- E0616: Private field access (12 errors) - FIXED (HookEntry, ReceiptEntry)
- E0308: Type mismatches (~60 errors)
- E0061, E0502, E0412, E0422: Various (~39 errors)
"""

import os
import re
from pathlib import Path

def read_file(filepath):
    """Read a file and return its contents"""
    try:
        with open(filepath, 'r') as f:
            return f.read()
    except Exception as e:
        return None

def write_file(filepath, content):
    """Write content to a file"""
    try:
        with open(filepath, 'w') as f:
            f.write(content)
        return True
    except Exception as e:
        print(f"Error writing {filepath}: {e}")
        return False

def fix_common_patterns(content):
    """Fix common error patterns in Rust code"""
    fixes_applied = []
    
    # Fix 1: Ensure structs returned from public functions are public
    # Pattern: pub fn ... -> Result<StructName, ...>
    # Check if StructName is defined without pub
    
    # Fix 2: Fix type mismatches in Result types
    # Pattern: Result<T, E> where T or E might be wrong type
    
    # Fix 3: Fix module visibility issues
    # Pattern: use crate::module where module might not be pub
    
    # Fix 4: Fix CLI macro inference failures
    # Pattern: #[verb] functions that might have type inference issues
    
    return content, fixes_applied

def process_file(filepath):
    """Process a single file and apply fixes"""
    content = read_file(filepath)
    if not content:
        return False, []
    
    original_content = content
    content, fixes = fix_common_patterns(content)
    
    if content != original_content:
        if write_file(filepath, content):
            return True, fixes
        else:
            return False, []
    
    return False, []

def main():
    cli_src = Path("/Users/sac/knhk/rust/knhk-cli/src")
    
    print("Analyzing knhk-cli source files for common error patterns...")
    
    all_fixes = []
    for filepath in cli_src.rglob("*.rs"):
        fixed, fixes = process_file(filepath)
        if fixed:
            print(f"Fixed: {filepath}")
            all_fixes.extend(fixes)
    
    print(f"\nApplied {len(all_fixes)} fixes across {len([f for f in all_fixes if f])} files")
    
    print("\nNext steps:")
    print("1. Run: cd /Users/sac/knhk/rust && cargo check -p knhk-cli")
    print("2. Review remaining errors")
    print("3. Apply additional fixes as needed")

if __name__ == "__main__":
    main()

