#!/bin/sh
# Pinned Linux solver for the Ubuntu 24.04 Kani jobs.
set -eu

test "$(uname -sm)" = "Linux x86_64"
test "$(getconf GNU_LIBC_VERSION)" = "glibc 2.39"
: "${GITHUB_PATH:?This installer requires a GitHub Actions environment}"

asset=z3-5.3.1-x64-glibc-2.39
sha256=acca9625e8cc8edeb9647b653a1bd696544ddeb5d1c2970a4576734a0c175b40
solver_cache="$HOME/.cache/kani-z3"
archive="$solver_cache/$asset.zip"
mkdir -p "$solver_cache"
if [ ! -f "$archive" ]; then
    gh release download z3-5.3.1 --repo Z3Prover/z3 --pattern "$asset.zip" --dir "$solver_cache"
fi
printf '%s  %s\n' "$sha256" "$archive" | sha256sum --check --strict
unzip -q -o "$archive" -d "$solver_cache"
solver_bin="$solver_cache/$asset/bin"
test "$("$solver_bin/z3" -version)" = "Z3 version 5.3.1 - 64 bit"
"$solver_bin/z3" -version
printf '%s\n' "$solver_bin" >> "$GITHUB_PATH"
