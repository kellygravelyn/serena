#!/usr/bin/env sh

# Script assumes it's being run at the root of the repository!

set -e
version="$1"

start_line=$(cat CHANGELOG.md | grep -n "^## $version" | cut -d : -f 1)
if [ -z "$start_line" ]; then
    echo "Version not found in CHANGELOG.md"
    exit 1
else
    start_line=$((start_line+2))
    end_line=$(tail -n +$start_line CHANGELOG.md | grep -m 1 -n '^##[[:space:]]' | cut -d : -f 1)
    
    if [ -z "$end_line" ]; then
        tail -n +$start_line CHANGELOG.md
    else
        end_line=$((end_line-2))
        if [ $end_line -ge 0 ]; then
            tail -n +$start_line CHANGELOG.md | head -n $end_line
        fi
    fi
fi
