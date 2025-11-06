#!/bin/bash
# Setup Git Hooks for KNHK

set -e

echo "Setting up Git hooks for KNHK..."

# Create symlink to pre-commit hook
ln -sf ../../scripts/pre-commit-hook.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

echo "✅ Git hooks installed successfully!"
echo ""
echo "Pre-commit hook will now run on every commit."
echo "To skip the hook (not recommended): git commit --no-verify"
