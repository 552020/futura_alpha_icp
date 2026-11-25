#!/bin/bash
# Do the committed interface files match the code?
#
#   scripts/check-did.sh          # exit 1 and show the difference if not
#   scripts/check-did.sh --fix    # rewrite the files from the code
#
# Needs no replica: the backend's interface is read from its wasm
# (candid-extractor), the factory's from its own export (an ignored test).
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
FIX="${1:-}"

command -v candid-extractor >/dev/null || {
    echo "candid-extractor missing: cargo install candid-extractor --locked"
    exit 1
}

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

cargo build --quiet --target wasm32-unknown-unknown --release -p backend
candid-extractor target/wasm32-unknown-unknown/release/backend.wasm > "$TMP/backend.did"

# The test prints the interface between the cargo test lines
cargo test --quiet -p canister_factory print_candid -- --ignored --nocapture 2>/dev/null \
    | sed -n '/^type\|^service/,$p' | sed '/^test tests::print_candid/,$d;/^\.$/,$d' \
    > "$TMP/canister_factory.did"

status=0
check() { # name, committed file
    if diff -q "$2" "$TMP/$1.did" >/dev/null; then
        echo "ok: $2"
    elif [ "$FIX" = "--fix" ]; then
        cp "$TMP/$1.did" "$2"
        echo "rewritten: $2"
    else
        echo "DIFFERENT: $2 (committed < > code)"
        diff "$2" "$TMP/$1.did" | head -40 || true
        status=1
    fi
}
check backend src/backend/backend.did
check canister_factory src/canister_factory/canister_factory.did
[ $status -eq 0 ] || echo "Run scripts/check-did.sh --fix and commit, then regenerate the web bindings."
exit $status
