# Quick Fix Instructions for Shell Issue

## Immediate Solution

The shell is broken due to Cursor configuration. Here's the fastest fix:

### Step 1: Open Terminal Outside Cursor
- Open Terminal.app (or iTerm, etc.) - NOT Cursor's terminal
- This bypasses Cursor's broken shell hooks

### Step 2: Fix the Configuration
```bash
# Check what's broken
grep -n "cursor_snap\|dump_zsh_state" ~/.zshrc

# Backup
cp ~/.zshrc ~/.zshrc.backup

# Comment out problematic lines
sed -i.bak 's/^[^#]*cursor_snap/# &/' ~/.zshrc
sed -i.bak 's/^[^#]*dump_zsh_state/# &/' ~/.zshrc

# Or manually edit:
nano ~/.zshrc
# Find lines with cursor_snap or dump_zsh_state
# Add # at the start of those lines
# Save and exit
```

### Step 3: Restart Terminal
```bash
# Reload shell config
source ~/.zshrc

# Or just close and reopen terminal
```

### Step 4: Verify It Works
```bash
echo "Shell works!"
cd /Users/sac/knhk/rust
cargo check --workspace
```

## Alternative: Use Bash Temporarily

If you can't fix zsh immediately:

```bash
# Switch to bash
chsh -s /bin/bash

# Restart terminal, then:
cd /Users/sac/knhk/rust
cargo check --workspace
```

## After Fix: Verify Compilation

```bash
cd /Users/sac/knhk

# Check Rust packages
cd rust
cargo check --workspace
cargo clippy --workspace -- -D warnings

# Check C code
cd ../c
make clean && make lib
```

