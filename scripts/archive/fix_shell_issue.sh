#!/bin/bash
# Script to fix shell issue by checking and fixing problematic Cursor configuration

set -e

echo "=== Checking for Cursor shell configuration issues ==="

# Check .zshrc
if [ -f ~/.zshrc ]; then
    echo ""
    echo "Checking ~/.zshrc for cursor_snap or dump_zsh_state..."
    if grep -q "cursor_snap\|dump_zsh_state" ~/.zshrc; then
        echo "FOUND: Problematic lines in ~/.zshrc:"
        grep -n "cursor_snap\|dump_zsh_state" ~/.zshrc
        echo ""
        echo "Backing up ~/.zshrc to ~/.zshrc.backup"
        cp ~/.zshrc ~/.zshrc.backup
        echo ""
        echo "Commenting out problematic lines..."
        sed -i.bak 's/^[^#]*cursor_snap/# &/' ~/.zshrc
        sed -i.bak 's/^[^#]*dump_zsh_state/# &/' ~/.zshrc
        echo "Fixed ~/.zshrc"
    else
        echo "No cursor_snap or dump_zsh_state found in ~/.zshrc"
    fi
fi

# Check .zshenv
if [ -f ~/.zshenv ]; then
    echo ""
    echo "Checking ~/.zshenv for cursor_snap or dump_zsh_state..."
    if grep -q "cursor_snap\|dump_zsh_state" ~/.zshenv; then
        echo "FOUND: Problematic lines in ~/.zshenv:"
        grep -n "cursor_snap\|dump_zsh_state" ~/.zshenv
        echo ""
        echo "Backing up ~/.zshenv to ~/.zshenv.backup"
        cp ~/.zshenv ~/.zshenv.backup
        echo ""
        echo "Commenting out problematic lines..."
        sed -i.bak 's/^[^#]*cursor_snap/# &/' ~/.zshenv
        sed -i.bak 's/^[^#]*dump_zsh_state/# &/' ~/.zshenv
        echo "Fixed ~/.zshenv"
    else
        echo "No cursor_snap or dump_zsh_state found in ~/.zshenv"
    fi
fi

# Check .zprofile
if [ -f ~/.zprofile ]; then
    echo ""
    echo "Checking ~/.zprofile for cursor_snap or dump_zsh_state..."
    if grep -q "cursor_snap\|dump_zsh_state" ~/.zprofile; then
        echo "FOUND: Problematic lines in ~/.zprofile:"
        grep -n "cursor_snap\|dump_zsh_state" ~/.zprofile
        echo ""
        echo "Backing up ~/.zprofile to ~/.zprofile.backup"
        cp ~/.zprofile ~/.zprofile.backup
        echo ""
        echo "Commenting out problematic lines..."
        sed -i.bak 's/^[^#]*cursor_snap/# &/' ~/.zprofile
        sed -i.bak 's/^[^#]*dump_zsh_state/# &/' ~/.zprofile
        echo "Fixed ~/.zprofile"
    else
        echo "No cursor_snap or dump_zsh_state found in ~/.zprofile"
    fi
fi

echo ""
echo "=== Fix complete ==="
echo "Please restart your terminal or run: source ~/.zshrc"

