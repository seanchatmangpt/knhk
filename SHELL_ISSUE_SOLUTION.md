# Shell Issue Solution

## Problem

The shell is failing during initialization with:
```
(eval):3: parse error near `cursor_snap_ENV_VARS...'
zsh:1: command not found: dump_zsh_state
```

This prevents all commands from running.

## Root Cause

Cursor has injected shell hooks that are broken:
- `cursor_snap_ENV_VARS` - causing a parse error
- `dump_zsh_state` - command not found

## Solution Options

### Option 1: Fix Shell Configuration (Recommended)

**Step 1**: Open a new terminal OUTSIDE of Cursor (Terminal.app, iTerm, etc.)

**Step 2**: Check for problematic lines:
```bash
grep -n "cursor_snap\|dump_zsh_state" ~/.zshrc ~/.zshenv ~/.zprofile
```

**Step 3**: Backup and fix the files:
```bash
# Backup
cp ~/.zshrc ~/.zshrc.backup
cp ~/.zshenv ~/.zshenv.backup 2>/dev/null || true
cp ~/.zprofile ~/.zprofile.backup 2>/dev/null || true

# Comment out problematic lines
sed -i.bak 's/^[^#]*cursor_snap/# &/' ~/.zshrc
sed -i.bak 's/^[^#]*dump_zsh_state/# &/' ~/.zshrc

# Or manually edit and comment out lines containing:
# - cursor_snap_ENV_VARS
# - dump_zsh_state
```

**Step 4**: Restart terminal or reload:
```bash
source ~/.zshrc
```

### Option 2: Use the Fix Script

**Step 1**: Open a new terminal OUTSIDE of Cursor

**Step 2**: Run the fix script:
```bash
cd /Users/sac/knhk
bash fix_shell_issue.sh
```

**Step 3**: Restart terminal

### Option 3: Use Clean Shell for Commands

**Step 1**: Open a new terminal OUTSIDE of Cursor

**Step 2**: Use the clean shell script:
```bash
cd /Users/sac/knhk
bash run_with_clean_shell.sh 'cd /Users/sac/knhk/rust && cargo check --workspace'
```

### Option 4: Temporarily Switch to Bash

**Step 1**: Open a new terminal OUTSIDE of Cursor

**Step 2**: Switch to bash:
```bash
chsh -s /bin/bash
```

**Step 3**: Restart terminal

**Step 4**: (Optional) Switch back later:
```bash
chsh -s /bin/zsh
```

## Quick Fix (Manual Edit)

1. Open Terminal.app (outside Cursor)
2. Edit `.zshrc`:
   ```bash
   nano ~/.zshrc
   # or
   vim ~/.zshrc
   ```
3. Find and comment out lines containing:
   - `cursor_snap_ENV_VARS`
   - `dump_zsh_state`
4. Save and restart terminal

## Verification

After fixing, verify the shell works:
```bash
echo "Shell works"
cd /Users/sac/knhk/rust
cargo check --workspace
```

## Files Created

1. **fix_shell_issue.sh** - Automated fix script
2. **run_with_clean_shell.sh** - Run commands with clean environment
3. **SHELL_ISSUE_SOLUTION.md** - This document

## Next Steps After Fix

Once the shell is fixed, run:
```bash
cd /Users/sac/knhk
bash verify_all_packages.sh
```

Or manually verify:
```bash
cd /Users/sac/knhk/rust
cargo check --workspace
cargo clippy --workspace -- -D warnings
cd ../c
make clean && make lib
```

