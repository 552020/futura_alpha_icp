#!/bin/bash
# An upgrade must keep uploaded files (blob store), not only the records.
# Regression: pre_upgrade used to snapshot over stable memory and the blob
# store was lost once the canister held more than a little data.
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_utils.sh"
cd "$SCRIPT_DIR/../../.."
WASM=target/wasm32-unknown-unknown/release/backend.wasm
[[ -f "$WASM" ]] || cargo build --target wasm32-unknown-unknown --release -p backend >/dev/null 2>&1
call() { dfx canister call backend "$@" 2>&1 | grep -v "^WARNING"; }

CAPSULE=$(call capsules_read_basic '(null)' | grep -o 'capsule_id = "[^"]*"' | head -1 | cut -d'"' -f2)
[[ -z "$CAPSULE" ]] && CAPSULE=$(call capsules_create '(null)' | grep -o 'id = "capsule_[^"]*"' | head -1 | cut -d'"' -f2)
DATA="file bytes that must survive an upgrade $RANDOM"
HASH=$(printf '%s' "$DATA" | sha256sum | cut -d' ' -f1 | sed 's/../\\&/g')
SID=$(call uploads_begin "(\"$CAPSULE\", 1 : nat32, \"upg-$RANDOM\")" | grep -oE "Ok = [0-9_]+" | grep -oE "[0-9_]+$")
call uploads_put_chunk "($SID : nat64, 0 : nat32, blob \"$DATA\")" >/dev/null
BLOB=$(call uploads_finish "($SID : nat64, blob \"$HASH\", ${#DATA} : nat64)" | grep -o 'blob_id = "[^"]*"' | cut -d'"' -f2)
[[ -n "$BLOB" ]] || { echo_error "upload failed"; exit 1; }
COUNT_BEFORE=$(call capsules_read_basic '(null)' | grep -o 'memory_count = [0-9_]*')

dfx canister install backend --mode upgrade --wasm "$WASM" --yes >/dev/null 2>&1 || { echo_error "upgrade failed"; exit 1; }

call blob_read "(\"$BLOB\")" | tr -s '\n ' ' ' | grep -q "$DATA" || { echo_error "the uploaded file is gone after the upgrade"; exit 1; }
echo_success "the uploaded file survives an upgrade"
[[ "$(call capsules_read_basic '(null)' | grep -o 'memory_count = [0-9_]*')" == "$COUNT_BEFORE" ]] || { echo_error "memories changed"; exit 1; }
echo_success "the records survive an upgrade ($COUNT_BEFORE)"
echo_success "✅ Upgrade test passed"
