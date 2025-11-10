#!/usr/bin/env python3
"""
Bulk Replace Print Statements with Tracing Logging

Replaces println!, eprintln!, print!, and eprint! statements with appropriate
tracing macros (tracing::error!, tracing::warn!, tracing::info!, tracing::debug!).

Usage:
    python3 scripts/replace_print_with_logging.py [options]

Options:
    --dry-run          Show changes without applying them
    --file <path>      Process specific file only
    --dir <path>       Process directory (default: rust/)
    --interactive      Prompt before each replacement
    --level <level>    Default log level (error|warn|info|debug|trace)
    --skip-tests       Skip test files
    --skip-examples    Skip example files
"""

import re
import sys
import argparse
from pathlib import Path
from typing import List, Tuple, Optional
from dataclasses import dataclass
from enum import Enum

class LogLevel(Enum):
    ERROR = "error"
    WARN = "warn"
    INFO = "info"
    DEBUG = "debug"
    TRACE = "trace"

@dataclass
class PrintStatement:
    """Represents a print statement to be replaced."""
    file_path: Path
    line_number: int
    original: str
    replacement: str
    log_level: LogLevel
    reason: str

class PrintReplacer:
    """Handles replacement of print statements with tracing macros."""
    
    # Patterns for different print macros
    # Match print macros with balanced parentheses
    PRINTLN_PATTERN = re.compile(r'println!\s*\((.*?)\);', re.DOTALL)
    EPRINTLN_PATTERN = re.compile(r'eprintln!\s*\((.*?)\);', re.DOTALL)
    PRINT_PATTERN = re.compile(r'print!\s*\((.*?)\);', re.DOTALL)
    EPRINT_PATTERN = re.compile(r'eprint!\s*\((.*?)\);', re.DOTALL)
    
    def find_balanced_content(self, text: str, start_pos: int) -> Optional[Tuple[int, int]]:
        """Find balanced parentheses content starting at start_pos."""
        if start_pos >= len(text) or text[start_pos] != '(':
            return None
        
        depth = 0
        i = start_pos
        start = i + 1  # Skip opening '('
        
        while i < len(text):
            if text[i] == '(':
                depth += 1
            elif text[i] == ')':
                depth -= 1
                if depth == 0:
                    return (start, i)  # Return (start, end) of content
            elif text[i] == '"':
                # Skip string literals
                i += 1
                while i < len(text) and text[i] != '"':
                    if text[i] == '\\':
                        i += 1  # Skip escaped character
                    i += 1
            i += 1
        
        return None  # Unbalanced parentheses
    
    # Patterns for determining log level from context
    ERROR_KEYWORDS = ['error', 'Error', 'ERROR', 'failed', 'Failed', 'FAILED', 
                      'failure', 'Failure', 'exception', 'Exception', 'panic']
    WARN_KEYWORDS = ['warning', 'Warning', 'WARNING', 'warn', 'Warn', 'WARN',
                     'gap', 'Gap', 'GAP', 'missing', 'Missing', 'deprecated']
    DEBUG_KEYWORDS = ['debug', 'Debug', 'DEBUG', 'test', 'Test', 'TEST',
                      '[TEST]', 'trace', 'Trace']
    
    def __init__(self, default_level: LogLevel = LogLevel.INFO, 
                 skip_tests: bool = False, skip_examples: bool = False):
        self.default_level = default_level
        self.skip_tests = skip_tests
        self.skip_examples = skip_examples
        self.replacements: List[PrintStatement] = []
    
    def should_skip_file(self, file_path: Path) -> bool:
        """Check if file should be skipped."""
        path_str = str(file_path)
        if self.skip_tests and ('/tests/' in path_str or '/test/' in path_str or file_path.name.endswith('_test.rs')):
            return True
        if self.skip_examples and '/examples/' in path_str:
            return True
        return False
    
    def determine_log_level(self, content: str, is_error: bool, 
                            context_lines: List[str]) -> LogLevel:
        """Determine appropriate log level from context."""
        # eprintln! defaults to warn/error
        if is_error:
            # Check for error keywords
            content_lower = content.lower()
            if any(keyword.lower() in content_lower for keyword in self.ERROR_KEYWORDS):
                return LogLevel.ERROR
            if any(keyword.lower() in content_lower for keyword in self.WARN_KEYWORDS):
                return LogLevel.WARN
            return LogLevel.WARN  # Default for eprintln!
        
        # Check content for keywords
        content_lower = content.lower()
        if any(keyword.lower() in content_lower for keyword in self.ERROR_KEYWORDS):
            return LogLevel.ERROR
        if any(keyword.lower() in content_lower for keyword in self.WARN_KEYWORDS):
            return LogLevel.WARN
        if any(keyword.lower() in content_lower for keyword in self.DEBUG_KEYWORDS):
            return LogLevel.DEBUG
        
        # Check context lines
        for line in context_lines:
            line_lower = line.lower()
            if any(keyword.lower() in line_lower for keyword in self.ERROR_KEYWORDS):
                return LogLevel.ERROR
            if any(keyword.lower() in line_lower for keyword in self.WARN_KEYWORDS):
                return LogLevel.WARN
        
        return self.default_level
    
    
    def convert_to_tracing(self, content: str, is_error: bool, 
                          context_lines: List[str], line_num: int) -> Tuple[str, LogLevel, str]:
        """Convert print statement content to tracing macro."""
        log_level = self.determine_log_level(content, is_error, context_lines)
        
        # Clean up content
        content = content.strip()
        
        # Handle simple string literals (no format args)
        if content.startswith('"') and content.endswith('"') and '{}' not in content:
            message = content.strip('"').replace('\\"', '"').replace('\\n', '\n')
            escaped_message = message.replace('"', '\\"')
            replacement = f'tracing::{log_level.value}!(\n        "{escaped_message}"\n    );'
            reason = f"Simple message ({'eprintln!' if is_error else 'println!'} -> tracing::{log_level.value}!)"
            return replacement, log_level, reason
        
        # Handle format! macro calls
        format_match = re.search(r'format!\s*\((.*?)\)', content, re.DOTALL)
        if format_match:
            format_content = format_match.group(1).strip()
            # Extract format string and args from format! call
            # Pattern: "format string", arg1, arg2
            string_match = re.match(r'^"([^"]*(?:\\.[^"]*)*)"', format_content)
            if string_match:
                format_str = string_match.group(1)
                remaining = format_content[len(string_match.group(0)):].strip()
                if remaining.startswith(','):
                    remaining = remaining[1:].strip()
                args = [arg.strip() for arg in remaining.split(',')] if remaining else []
                
                # Build replacement
                if args:
                    field_count = format_str.count('{}')
                    if field_count == len(args):
                        # Extract field names from args
                        fields = []
                        for arg in args:
                            var_match = re.match(r'(\w+)', arg.strip())
                            if var_match:
                                fields.append(f"{var_match.group(1)} = {arg.strip()}")
                            else:
                                fields.append(f"arg = {arg.strip()}")
                        
                        # Create message without placeholders
                        message = format_str.replace('{}', '{}')
                        escaped_message = message.replace('"', '\\"')
                        fields_str = ',\n        '.join(fields)
                        replacement = f'tracing::{log_level.value}!(\n        {fields_str},\n        "{escaped_message}"\n    );'
                    else:
                        # Mismatch - use format! wrapper
                        replacement = f'tracing::{log_level.value}!(\n        message = format!({format_content}),\n        "Log message"\n    );'
                else:
                    # No args
                    escaped_message = format_str.replace('"', '\\"')
                    replacement = f'tracing::{log_level.value}!(\n        "{escaped_message}"\n    );'
            else:
                # Fallback
                replacement = f'tracing::{log_level.value}!(\n        message = format!({format_content}),\n        "Log message"\n    );'
            
            reason = f"Format macro ({'eprintln!' if is_error else 'println!'} -> tracing::{log_level.value}!)"
            return replacement, log_level, reason
        
        # Handle format string with args: "message {}", arg
        string_match = re.match(r'^"([^"]*(?:\\.[^"]*)*)"', content)
        if string_match:
            format_str = string_match.group(1)
            remaining = content[len(string_match.group(0)):].strip()
            if remaining.startswith(','):
                remaining = remaining[1:].strip()
            args = [arg.strip() for arg in remaining.split(',')] if remaining else []
            
            if args:
                # Count {} placeholders
                field_count = format_str.count('{}')
                if field_count == len(args):
                    # Extract field names from args
                    fields = []
                    for arg in args:
                        # Try to get variable name
                        var_match = re.match(r'(\w+)', arg.strip())
                        if var_match:
                            var_name = var_match.group(1)
                            # Use % for Display, ? for Debug
                            if 'error' in format_str.lower() or 'Error' in format_str or 'failed' in format_str.lower():
                                fields.append(f"error.message = %{arg.strip()}")
                            elif var_name.lower() in ['e', 'err', 'error']:
                                fields.append(f"error.message = %{arg.strip()}")
                            else:
                                # Try to infer field name from format string context
                                # Look for patterns like "Processing {} triples" -> triple_count
                                if 'count' in format_str.lower() or 'number' in format_str.lower():
                                    fields.append(f"count = {arg.strip()}")
                                elif 'id' in format_str.lower():
                                    fields.append(f"id = %{arg.strip()}")
                                else:
                                    fields.append(f"{var_name} = {arg.strip()}")
                        else:
                            fields.append(f"arg = {arg.strip()}")
                    
                    # Create cleaner message without {} placeholders
                    message_parts = format_str.split('{}')
                    if len(message_parts) == 2:
                        # Simple case: "prefix {} suffix" -> "prefix suffix"
                        message = f"{message_parts[0].strip()} {message_parts[1].strip()}".strip()
                    else:
                        # Multiple placeholders - just remove {}
                        message = format_str.replace('{}', '')
                    
                    # Clean up message
                    message = re.sub(r'\s+', ' ', message).strip()
                    escaped_message = message.replace('"', '\\"')
                    fields_str = ',\n        '.join(fields)
                    replacement = f'tracing::{log_level.value}!(\n        {fields_str},\n        "{escaped_message}"\n    );'
                else:
                    # Mismatch - use format! wrapper
                    replacement = f'tracing::{log_level.value}!(\n        message = format!({content}),\n        "Log message"\n    );'
            else:
                # No args - simple message
                escaped_message = format_str.replace('"', '\\"')
                replacement = f'tracing::{log_level.value}!(\n        "{escaped_message}"\n    );'
            
            reason = f"Format string ({'eprintln!' if is_error else 'println!'} -> tracing::{log_level.value}!)"
            return replacement, log_level, reason
        
        # Fallback: wrap entire content in format!
        replacement = f'tracing::{log_level.value}!(\n        message = format!({content}),\n        "Log message"\n    );'
        reason = f"Fallback conversion ({'eprintln!' if is_error else 'println!'} -> tracing::{log_level.value}!)"
        return replacement, log_level, reason
    
    def process_file(self, file_path: Path) -> List[PrintStatement]:
        """Process a single file and find all print statements."""
        if self.should_skip_file(file_path):
            return []
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                lines = f.readlines()
        except Exception as e:
            print(f"Error reading {file_path}: {e}", file=sys.stderr)
            return []
        
        replacements = []
        content = ''.join(lines)
        
        # Find all print statements
        for pattern, is_error in [
            (self.EPRINTLN_PATTERN, True),
            (self.PRINTLN_PATTERN, False),
            (self.EPRINT_PATTERN, True),
            (self.PRINT_PATTERN, False),
        ]:
            for match in pattern.finditer(content):
                start_pos = match.start()
                line_num = content[:start_pos].count('\n') + 1
                
                # Get context lines (3 before, 3 after)
                context_start = max(0, line_num - 4)
                context_end = min(len(lines), line_num + 3)
                context_lines = [lines[i].strip() for i in range(context_start, context_end)]
                
                # Convert to tracing
                replacement, log_level, reason = self.convert_to_tracing(
                    match.group(1), is_error, context_lines, line_num
                )
                
                replacements.append(PrintStatement(
                    file_path=file_path,
                    line_number=line_num,
                    original=match.group(0),
                    replacement=replacement,
                    log_level=log_level,
                    reason=reason
                ))
        
        return replacements
    
    def apply_replacements(self, file_path: Path, replacements: List[PrintStatement], 
                          dry_run: bool = False, interactive: bool = False) -> bool:
        """Apply replacements to a file."""
        if not replacements:
            return True
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            print(f"Error reading {file_path}: {e}", file=sys.stderr)
            return False
        
        # Sort replacements by line number (reverse to maintain positions)
        replacements_sorted = sorted(replacements, key=lambda x: x.line_number, reverse=True)
        
        modified_content = content
        for replacement in replacements_sorted:
            if interactive:
                print(f"\n{file_path}:{replacement.line_number}")
                print(f"  Original: {replacement.original}")
                print(f"  Replacement: {replacement.replacement}")
                print(f"  Level: {replacement.log_level.value}")
                print(f"  Reason: {replacement.reason}")
                response = input("  Apply? [y/N]: ").strip().lower()
                if response != 'y':
                    continue
            
            # Replace in content
            modified_content = modified_content.replace(replacement.original, replacement.replacement, 1)
        
        if dry_run:
            # Show diff
            if modified_content != content:
                print(f"\n=== {file_path} ===")
                # Simple diff display
                original_lines = content.split('\n')
                modified_lines = modified_content.split('\n')
                for i, (orig, mod) in enumerate(zip(original_lines, modified_lines), 1):
                    if orig != mod:
                        print(f"  {i:-} {orig}")
                        print(f"  {i:+} {mod}")
            return True
        
        # Write modified content
        try:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(modified_content)
            return True
        except Exception as e:
            print(f"Error writing {file_path}: {e}", file=sys.stderr)
            return False

def main():
    parser = argparse.ArgumentParser(
        description='Replace print statements with tracing logging macros'
    )
    parser.add_argument('--dry-run', action='store_true',
                       help='Show changes without applying them')
    parser.add_argument('--file', type=str,
                       help='Process specific file only')
    parser.add_argument('--dir', type=str, default='rust/',
                       help='Process directory (default: rust/)')
    parser.add_argument('--interactive', action='store_true',
                       help='Prompt before each replacement')
    parser.add_argument('--level', type=str, default='info',
                       choices=['error', 'warn', 'info', 'debug', 'trace'],
                       help='Default log level (default: info)')
    parser.add_argument('--skip-tests', action='store_true',
                       help='Skip test files')
    parser.add_argument('--skip-examples', action='store_true',
                       help='Skip example files')
    
    args = parser.parse_args()
    
    # Convert level string to enum
    default_level = LogLevel(args.level)
    
    replacer = PrintReplacer(
        default_level=default_level,
        skip_tests=args.skip_tests,
        skip_examples=args.skip_examples
    )
    
    # Find files to process
    if args.file:
        files = [Path(args.file)]
    else:
        dir_path = Path(args.dir)
        if not dir_path.exists():
            print(f"Error: Directory {dir_path} does not exist", file=sys.stderr)
            return 1
        
        files = list(dir_path.rglob('*.rs'))
    
    # Process files
    total_replacements = 0
    files_modified = 0
    
    for file_path in files:
        replacements = replacer.process_file(file_path)
        if replacements:
            total_replacements += len(replacements)
            if replacer.apply_replacements(file_path, replacements, 
                                         dry_run=args.dry_run, 
                                         interactive=args.interactive):
                if not args.dry_run:
                    files_modified += 1
                    print(f"✓ Processed {file_path} ({len(replacements)} replacements)")
                else:
                    print(f"Would process {file_path} ({len(replacements)} replacements)")
    
    # Summary
    print(f"\n=== Summary ===")
    print(f"Total replacements found: {total_replacements}")
    if args.dry_run:
        print(f"Run without --dry-run to apply changes")
    else:
        print(f"Files modified: {files_modified}")
    
    return 0

if __name__ == '__main__':
    sys.exit(main())

