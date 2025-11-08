#!/usr/bin/env python3
"""
Extract code blocks from yawl.txt and save them to separate files.
Handles both language markers and file path markers.
"""

import re
import os
from pathlib import Path

def detect_language_from_filename(filename):
    """Detect language from filename extension."""
    ext_map = {
        '.rs': 'rust',
        '.c': 'c',
        '.h': 'c',
        '.toml': 'toml',
        '.xml': 'xml',
        '.ttl': 'turtle',
        '.json': 'json',
        '.yaml': 'yaml',
        '.yml': 'yaml',
        '.sh': 'bash',
        '.bash': 'bash',
        '.py': 'python',
        '.js': 'javascript',
        '.ts': 'typescript',
        '.mk': 'makefile',
        '.md': 'text',
        '.txt': 'text',
    }
    
    for ext, lang in ext_map.items():
        if filename.endswith(ext):
            return lang
    return None

def extract_code_blocks(file_path, output_dir):
    """Extract all code blocks from the file."""
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)
    
    with open(file_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    code_blocks = []
    current_lang = None
    current_code = []
    current_start_line = None
    current_filename = None
    block_counter = {}
    
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        
        # Check if this is a language identifier (common code languages)
        lang_patterns = [
            r'^rust$', r'^c$', r'^toml$', r'^xml$', r'^turtle$', r'^json$',
            r'^yaml$', r'^bash$', r'^sh$', r'^python$', r'^javascript$',
            r'^typescript$', r'^makefile$', r'^text$', r'^plain$'
        ]
        
        is_lang = False
        lang_name = None
        for pattern in lang_patterns:
            if re.match(pattern, stripped, re.IGNORECASE):
                is_lang = True
                lang_name = stripped.lower()
                break
        
        # Check if this is a file path marker (e.g., "Root Cargo.toml", "src/lib.rs")
        is_file_path = False
        file_path = None
        if not is_lang and stripped:
            # Patterns like "Root Cargo.toml", "src/lib.rs", "crates/patterns/Cargo.toml"
            file_path_patterns = [
                r'^[A-Z][a-zA-Z\s]+\.(rs|c|h|toml|xml|ttl|json|yaml|yml|sh|py|js|ts|mk|md|txt)$',
                r'^[a-z_/]+\.(rs|c|h|toml|xml|ttl|json|yaml|yml|sh|py|js|ts|mk|md|txt)$',
                r'^[a-z_/]+/[a-z_/]+\.(rs|c|h|toml|xml|ttl|json|yaml|yml|sh|py|js|ts|mk|md|txt)$',
            ]
            
            for pattern in file_path_patterns:
                match = re.search(r'([a-zA-Z_/]+\.(rs|c|h|toml|xml|ttl|json|yaml|yml|sh|py|js|ts|mk|md|txt))$', stripped)
                if match:
                    is_file_path = True
                    file_path = match.group(1)
                    lang_name = detect_language_from_filename(file_path)
                    if lang_name:
                        break
        
        if is_lang or (is_file_path and lang_name):
            # Save previous block if exists
            if current_lang and current_code:
                code_blocks.append({
                    'lang': current_lang,
                    'code': ''.join(current_code),
                    'start_line': current_start_line,
                    'end_line': i - 1,
                    'filename': current_filename
                })
            
            # Start new block
            current_lang = lang_name
            current_code = []
            current_start_line = i + 1
            current_filename = file_path if is_file_path else None
            i += 1
            continue
        
        # Check if we're in a code block
        if current_lang:
            # Check if this line ends the code block
            # Code blocks typically end with an empty line followed by non-indented text
            # or a new file path marker
            if stripped == '' and i + 1 < len(lines):
                next_line = lines[i + 1].strip()
                # If next line is empty or starts a new section/file, end the block
                if next_line == '':
                    # Check if there's another empty line or a new marker after
                    if i + 2 < len(lines):
                        next_next = lines[i + 2].strip()
                        if next_next == '' or (next_next and not next_next.startswith(' ') and not next_next.startswith('\t')):
                            if current_code:  # Only save if we have code
                                code_blocks.append({
                                    'lang': current_lang,
                                    'code': ''.join(current_code),
                                    'start_line': current_start_line,
                                    'end_line': i,
                                    'filename': current_filename
                                })
                            current_lang = None
                            current_code = []
                            current_filename = None
                            i += 1
                            continue
                elif next_line and not next_line.startswith(' ') and not next_line.startswith('\t'):
                    # Check if it's a new file path or language marker
                    next_is_file = re.search(r'([a-zA-Z_/]+\.(rs|c|h|toml|xml|ttl|json|yaml|yml|sh|py|js|ts|mk|md|txt))$', next_line)
                    next_is_lang = any(re.match(p, next_line, re.IGNORECASE) for p in lang_patterns)
                    if next_is_file or next_is_lang:
                        if current_code:  # Only save if we have code
                            code_blocks.append({
                                'lang': current_lang,
                                'code': ''.join(current_code),
                                'start_line': current_start_line,
                                'end_line': i,
                                'filename': current_filename
                            })
                        current_lang = None
                        current_code = []
                        current_filename = None
                        i += 1
                        continue
            
            # Add line to current code block
            current_code.append(line)
        
        i += 1
    
    # Save last block if exists
    if current_lang and current_code:
        code_blocks.append({
            'lang': current_lang,
            'code': ''.join(current_code),
            'start_line': current_start_line,
            'end_line': len(lines) - 1,
            'filename': current_filename
        })
    
    # Write code blocks to files
    for block in code_blocks:
        lang = block['lang']
        code = block['code'].rstrip()  # Remove trailing whitespace
        filename_hint = block.get('filename')
        
        # Skip empty blocks
        if not code.strip():
            continue
        
        # Generate filename
        if lang not in block_counter:
            block_counter[lang] = 0
        block_counter[lang] += 1
        
        # Use filename hint if available, otherwise generate
        if filename_hint:
            # Clean up filename (remove path separators, keep just the name)
            safe_name = filename_hint.replace('/', '_').replace('\\', '_')
            filename = f"{safe_name}"
        else:
            # Determine file extension
            ext_map = {
                'rust': 'rs',
                'c': 'c',
                'toml': 'toml',
                'xml': 'xml',
                'turtle': 'ttl',
                'json': 'json',
                'yaml': 'yaml',
                'bash': 'sh',
                'sh': 'sh',
                'python': 'py',
                'javascript': 'js',
                'typescript': 'ts',
                'makefile': 'mk',
                'text': 'txt',
                'plain': 'txt'
            }
            ext = ext_map.get(lang, 'txt')
            filename = f"{lang}_{block_counter[lang]:03d}.{ext}"
        
        filepath = output_path / filename
        
        # Avoid overwriting - add counter if file exists
        if filepath.exists() and not filename_hint:
            filename = f"{lang}_{block_counter[lang]:03d}.{ext_map.get(lang, 'txt')}"
            filepath = output_path / filename
        
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(code)
        
        print(f"Extracted {lang} block {block_counter[lang]} ({block['start_line']}-{block['end_line']}) -> {filename}")
    
    print(f"\nTotal code blocks extracted: {sum(block_counter.values())}")
    return code_blocks

if __name__ == '__main__':
    input_file = '/Users/sac/knhk/rust/docs/yawl/yawl.txt'
    output_dir = '/Users/sac/knhk/rust/docs/yawl/code'
    extract_code_blocks(input_file, output_dir)

