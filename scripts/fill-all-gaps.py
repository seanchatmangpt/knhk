#!/usr/bin/env python3
"""
Fill All Gaps - Direct File Verification and Fixing
Reads files directly to verify and fix issues
"""

import os
import re
import sys
from pathlib import Path

def find_rust_files(root_dir):
    """Find all Rust source files"""
    rust_files = []
    for path in Path(root_dir).rglob("*.rs"):
        if path.is_file() and "target" not in str(path) and "src" in str(path):
            rust_files.append(path)
    return sorted(rust_files)

def find_test_files(root_dir):
    """Find all Chicago TDD test files"""
    test_files = []
    for path in Path(root_dir).rglob("*chicago_tdd*.rs"):
        if path.is_file():
            test_files.append(path)
    return sorted(test_files)

def check_unwrap_expect(file_path):
    """Check for unwrap()/expect() in production code"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Check for unwrap() or expect() (excluding tests)
        if "tests" not in str(file_path) and "test" not in str(file_path):
            unwrap_count = content.count('.unwrap()')
            expect_count = len(re.findall(r'\.expect\([^)]*\)', content))
            if unwrap_count > 0 or expect_count > 0:
                return True, unwrap_count + expect_count
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
    return False, 0

def check_async_trait(file_path):
    """Check for async trait methods"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Check for async fn in trait or trait with async fn
        if re.search(r'trait\s+\w+.*\{[^}]*async\s+fn', content, re.DOTALL):
            return True
        if re.search(r'async\s+fn.*trait', content):
            return True
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
    return False

def check_unimplemented(file_path):
    """Check for unimplemented!() in production code"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()
        
        for i, line in enumerate(lines, 1):
            if 'unimplemented!' in line and not line.strip().startswith('//'):
                # Check if it's in a comment
                if '//' in line and line.find('//') < line.find('unimplemented!'):
                    continue
                return True, i
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
    return False, 0

def check_panic(file_path):
    """Check for panic!() in production code"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        if 'panic!' in content:
            # Count occurrences
            count = content.count('panic!')
            return True, count
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
    return False, 0

def check_uppercase_variables(file_path):
    """Check for uppercase S, P, O variables"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()
        
        issues = []
        for i, line in enumerate(lines, 1):
            # Check for let S =, let P =, let O = (not in comments or allow attributes)
            if re.search(r'\blet\s+S\s*=', line) and '#[allow' not in '\n'.join(lines[max(0,i-3):i]):
                issues.append((i, 'let S ='))
            if re.search(r'\blet\s+P\s*=', line) and '#[allow' not in '\n'.join(lines[max(0,i-3):i]):
                issues.append((i, 'let P ='))
            if re.search(r'\blet\s+O\s*=', line) and '#[allow' not in '\n'.join(lines[max(0,i-3):i]):
                issues.append((i, 'let O ='))
        
        return issues
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
    return []

def main():
    root_dir = Path(__file__).parent.parent
    rust_dir = root_dir / "rust"
    
    if not rust_dir.exists():
        print(f"Error: {rust_dir} does not exist")
        sys.exit(1)
    
    print("=== Filling All Gaps - Direct File Verification ===")
    print(f"Searching in: {rust_dir}")
    print()
    
    # 1. Verify Chicago TDD test files exist
    print("=== Gap 1: Chicago TDD Test Files ===")
    test_files = find_test_files(rust_dir)
    print(f"Found {len(test_files)} Chicago TDD test files:")
    for f in test_files:
        print(f"  ✓ {f.relative_to(root_dir)}")
    print()
    
    # 2. Check for unwrap()/expect() in production code
    print("=== Gap 2: Unwrap()/Expect() in Production Code ===")
    rust_files = find_rust_files(rust_dir)
    unwrap_files = []
    for f in rust_files:
        if "test" not in str(f) and "tests" not in str(f):
            has_issue, count = check_unwrap_expect(f)
            if has_issue:
                unwrap_files.append((f, count))
    
    if unwrap_files:
        print(f"⚠️  Found unwrap()/expect() in {len(unwrap_files)} production files:")
        for f, count in unwrap_files[:10]:
            print(f"  - {f.relative_to(root_dir)} ({count} occurrences)")
    else:
        print("✅ PASSED: No unwrap()/expect() in production code")
    print()
    
    # 3. Check for async trait methods
    print("=== Gap 3: Async Trait Methods ===")
    async_trait_files = []
    for f in rust_files:
        if check_async_trait(f):
            async_trait_files.append(f)
    
    if async_trait_files:
        print(f"❌ FAILED: Found async trait methods in {len(async_trait_files)} files:")
        for f in async_trait_files[:10]:
            print(f"  - {f.relative_to(root_dir)}")
    else:
        print("✅ PASSED: No async trait methods found")
    print()
    
    # 4. Check for unimplemented!()
    print("=== Gap 4: Unimplemented!() Check ===")
    unimplemented_files = []
    for f in rust_files:
        if "test" not in str(f) and "tests" not in str(f):
            has_issue, line = check_unimplemented(f)
            if has_issue:
                unimplemented_files.append((f, line))
    
    if unimplemented_files:
        print(f"⚠️  Found unimplemented!() in {len(unimplemented_files)} production files:")
        for f, line in unimplemented_files[:10]:
            print(f"  - {f.relative_to(root_dir)} (line {line})")
    else:
        print("✅ PASSED: No unimplemented!() in production code")
    print()
    
    # 5. Check for panic!()
    print("=== Gap 5: Panic!() Check ===")
    panic_files = []
    for f in rust_files:
        if "test" not in str(f) and "tests" not in str(f):
            has_issue, count = check_panic(f)
            if has_issue:
                panic_files.append((f, count))
    
    if panic_files:
        print(f"⚠️  Found panic!() in {len(panic_files)} production files:")
        for f, count in panic_files[:10]:
            print(f"  - {f.relative_to(root_dir)} ({count} occurrences)")
    else:
        print("✅ PASSED: No panic!() in production code")
    print()
    
    # 6. Check for uppercase variables
    print("=== Gap 6: Uppercase Variables Check ===")
    uppercase_issues = []
    for f in rust_files:
        issues = check_uppercase_variables(f)
        if issues:
            uppercase_issues.append((f, issues))
    
    if uppercase_issues:
        print(f"❌ FAILED: Found uppercase variables in {len(uppercase_issues)} files:")
        for f, issues in uppercase_issues[:5]:
            print(f"  - {f.relative_to(root_dir)}:")
            for line_num, var in issues[:3]:
                print(f"    Line {line_num}: {var}")
    else:
        print("✅ PASSED: No uppercase S, P, O variables found")
    print()
    
    # Summary
    print("=== Summary ===")
    total_issues = len(unwrap_files) + len(async_trait_files) + len(unimplemented_files) + len(panic_files) + len(uppercase_issues)
    if total_issues == 0:
        print("✅ ALL GAPS FILLED - No issues found!")
        return 0
    else:
        print(f"⚠️  Found {total_issues} categories of issues to review")
        return 1

if __name__ == "__main__":
    sys.exit(main())

