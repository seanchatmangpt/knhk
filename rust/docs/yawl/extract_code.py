#!/usr/bin/env python3
"""
Extract code blocks from yawl.txt and save them to separate files.
Code blocks are identified by language names on their own lines followed by code.
"""

import re
import os
from pathlib import Path

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
        
        if is_lang:
            # Save previous block if exists
            if current_lang and current_code:
                code_blocks.append({
                    'lang': current_lang,
                    'code': ''.join(current_code),
                    'start_line': current_start_line,
                    'end_line': i - 1
                })
            
            # Start new block
            current_lang = lang_name
            current_code = []
            current_start_line = i + 1
            i += 1
            continue
        
        # Check if we're in a code block
        if current_lang:
            # Check if this line ends the code block (empty line or next section)
            # Code blocks typically end with an empty line followed by non-indented text
            if stripped == '' and i + 1 < len(lines):
                next_line = lines[i + 1].strip()
                # If next line is empty or starts a new section, end the block
                if next_line == '' or (next_line and not next_line.startswith(' ') and not next_line.startswith('\t')):
                    if current_code:  # Only save if we have code
                        code_blocks.append({
                            'lang': current_lang,
                            'code': ''.join(current_code),
                            'start_line': current_start_line,
                            'end_line': i
                        })
                    current_lang = None
                    current_code = []
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
            'end_line': len(lines) - 1
        })
    
    # Write code blocks to files
    for block in code_blocks:
        lang = block['lang']
        code = block['code'].rstrip()  # Remove trailing whitespace
        
        # Skip empty blocks
        if not code.strip():
            continue
        
        # Generate filename
        if lang not in block_counter:
            block_counter[lang] = 0
        block_counter[lang] += 1
        
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
        
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(code)
        
        print(f"Extracted {lang} block {block_counter[lang]} ({block['start_line']}-{block['end_line']}) -> {filename}")
    
    print(f"\nTotal code blocks extracted: {sum(block_counter.values())}")
    return code_blocks

if __name__ == '__main__':
    input_file = '/Users/sac/knhk/rust/docs/yawl/yawl.txt'
    output_dir = '/Users/sac/knhk/rust/docs/yawl/code'
    extract_code_blocks(input_file, output_dir)

