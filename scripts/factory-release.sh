#!/bin/bash
# Release the backend wasm to the factory, so that create_personal_canister
# installs it into new personal canisters.
#
#   scripts/factory-release.sh <version> [dfx network, default local]
#
# Builds the backend, gzips it (the IC accepts gzipped modules; the plain wasm
# is above the 2 MB message limit), uploads it in chunks with the current dfx
# identity (which must be a factory admin) and calls release_set.
set -euo pipefail
VERSION="${1:?usage: factory-release.sh <version> [network]}"
NETWORK="${2:-local}"
cd "$(dirname "${BASH_SOURCE[0]}")/.."

cargo build --target wasm32-unknown-unknown --release -p backend
WASM=target/wasm32-unknown-unknown/release/backend.wasm
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
gzip -9 -n -c "$WASM" > "$TMP/backend.wasm.gz"
SHA=$(sha256sum "$TMP/backend.wasm.gz" | cut -d' ' -f1)
echo "backend.wasm.gz: $(stat -c %s "$TMP/backend.wasm.gz") bytes, sha256 $SHA"

call() { dfx canister call --network "$NETWORK" canister_factory "$@" 2>&1 | grep -v '^WARNING'; }

UPLOAD=$(call create_upload | grep -oE '[0-9_]+ : nat64' | head -1 | tr -d '_ ' | cut -d: -f1)
[[ -n "$UPLOAD" ]] || { echo "create_upload failed"; exit 1; }

split -b 900000 -d "$TMP/backend.wasm.gz" "$TMP/chunk."
for f in "$TMP"/chunk.*; do
    python3 - "$f" "$UPLOAD" > "$TMP/arg.did" <<'PY'
import sys
data = open(sys.argv[1], 'rb').read()
print('(%s : nat64, blob "%s")' % (sys.argv[2], ''.join('\\%02x' % b for b in data)))
PY
    call put_chunk --argument-file "$TMP/arg.did" | grep -q "Ok" || { echo "put_chunk failed for $f"; exit 1; }
done
call commit_upload "($UPLOAD : nat64, record { expected_sha256_hex = \"$SHA\" })" | grep -q "Ok" || { echo "commit_upload failed"; exit 1; }
call release_set "($UPLOAD : nat64, \"$VERSION\")"
