#!/usr/bin/env sh

# Script assumes it's being run at the root of the repository!

set -e
new_version="$1"

if [ -z "$(git status --porcelain)" ]; then
    if [ -z "$(./scripts/gen-release-notes.sh Unreleased)" ]; then
        echo "There is nothing in the Unreleased section of CHANGELOG.md."
        echo "Add changes to CHANGELOG.md and try again."
    else
        echo "Updating CHANGELOG.md…"
        sed "s/## Unreleased/## Unreleased\n\n## $new_version - $(date +'%Y-%m-%d')/" CHANGELOG.md > CHANGELOG.md.tmp
        mv CHANGELOG.md.tmp CHANGELOG.md

        echo "Updating Cargo.toml…"
        sed "s/^version = .*/version = \"$new_version\"/" Cargo.toml > Cargo.toml.tmp
        mv Cargo.toml.tmp Cargo.toml

        echo "Updating Cargo.lock with new workspace version…"
        cargo update --workspace

        echo "Ensuring project can still build debug and release…"
        cargo build
        cargo build --release

        echo "Creating git commit and tag…"
        git add CHANGELOG.md Cargo.toml Cargo.lock
        git commit -m "$new_version"
        git tag -a "v$new_version" -m "Release $new_version"

        echo "Done!"
    fi
else
    echo "Git repository needs to be in a clean state to prepare a release."
    echo "Commit or revert any changes and try again."
fi
