#!/usr/bin/env bash

set -ex

cargo install convco
cargo install cargo-release

convco changelog > CHANGELOG.md
git commit -am "chore(release): update changelog"

# shellcheck disable=SC2068
cargo release "$(convco version --bump)" --skip-publish --execute $@
