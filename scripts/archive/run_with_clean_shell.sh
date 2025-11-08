#!/bin/bash
# Run commands with a clean shell environment, bypassing broken zsh config

# This script runs commands in a clean bash environment without loading zsh config

if [ $# -eq 0 ]; then
    echo "Usage: $0 <command>"
    echo "Example: $0 'cd /Users/sac/knhk/rust && cargo check --workspace'"
    exit 1
fi

# Run command in clean bash environment
/usr/bin/env -i \
    PATH=/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin \
    HOME=$HOME \
    USER=$USER \
    /bin/bash --norc --noprofile -c "$@"

