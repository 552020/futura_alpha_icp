#!/bin/bash
# Personal canisters end to end (Juno-style): the factory creates a canister for
# a user, installs the released backend, and removes itself as controller.
# Then: only the owner can store data there, and the owner upgrades it alone.
# Needs the local replica with canister_factory deployed (dfx deploy canister_factory).
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$SCRIPT_DIR/../../.."
source "$SCRIPT_DIR/../test_utils.sh"
cd "$ROOT"

fail() { echo_error "$1"; exit 1; }
expect() { echo "$2" | tr -s '\n ' ' ' | grep -q "$3" || fail "$1: expected '$3', got: $2"; echo_success "$1"; }
q() { grep -v "^WARNING"; }

USER_ID="futura-test-owner-$RANDOM"   # a fresh user each run: one canister per user
OTHER_ID=futura-test-heir
dfx identity new "$USER_ID" --storage-mode plaintext >/dev/null 2>&1
dfx identity list 2>/dev/null | grep -qx "$OTHER_ID" || dfx identity new "$OTHER_ID" --storage-mode plaintext >/dev/null 2>&1
trap 'dfx identity remove "$USER_ID" >/dev/null 2>&1 || true' EXIT
USER_P=$(dfx identity get-principal --identity "$USER_ID")
FACTORY=$(dfx canister id canister_factory)

# The factory needs cycles and a release (admin = the default identity)
dfx ledger fabricate-cycles --canister canister_factory --t 20 >/dev/null 2>&1 || true
if dfx canister call canister_factory release_info 2>&1 | q | grep -q "(null)"; then
    echo_info "No release yet: releasing the backend wasm"
    scripts/factory-release.sh 0.0.0-test >/dev/null
fi
RELEASE_SHA=$(dfx canister call canister_factory release_info 2>&1 | q | grep -o 'sha256_hex = "[^"]*"' | cut -d'"' -f2)

r=$(dfx canister call --identity anonymous canister_factory create_personal_canister 2>&1 | q)
expect "anonymous callers get no canister" "$r" "anonymous"

r=$(dfx canister call --identity "$USER_ID" canister_factory create_personal_canister 2>&1 | q)
expect "the factory creates the user's canister" "$r" "status = variant { Ready }"
PC=$(echo "$r" | grep -oE 'canister_id = opt principal "[^"]+"' | cut -d'"' -f2)

r=$(dfx canister call --identity "$USER_ID" canister_factory create_personal_canister 2>&1 | q)
expect "asking again returns the same canister" "$r" "$PC"

st=$(dfx canister status --identity "$USER_ID" "$PC" 2>&1)
[[ "$(echo "$st" | grep Controllers | xargs)" == "Controllers: $USER_P" ]] || fail "controllers are not exactly the user: $(echo "$st" | grep Controllers)"
echo_success "the user is the only controller (the factory removed itself)"
expect "the installed code is the release" "$st" "0x$RELEASE_SHA"
r=$(dfx canister status "$PC" 2>&1 || true)
expect "the factory's admin cannot even read its status" "$r" "nly.*controllers"

pc() { local who="$1"; shift; dfx canister call --identity "$who" --candid src/backend/backend.did "$PC" "$@" 2>&1 | q; }
r=$(pc "$USER_ID" canister_mode)
expect "it runs in personal mode for this user" "$r" "Personal = record { owner = principal \"$USER_P\""

r=$(pc "$OTHER_ID" capsules_create '(null)')
expect "another user cannot create a capsule in it" "$r" "Unauthorized"
r=$(pc "$USER_ID" capsules_create '(null)')
expect "the owner creates their capsule" "$r" "Ok ="
CAPSULE=$(echo "$r" | grep -o 'id = "capsule_[^"]*"' | head -1 | cut -d'"' -f2)

META='(variant { Note = record { base = record { name = "first"; description = null; tags = vec {}; asset_type = variant { Original }; bytes = 5; mime_type = "text/plain"; sha256 = null; width = null; height = null; url = null; storage_key = null; bucket = null; asset_location = null; processing_status = null; processing_error = null; created_at = 0; updated_at = 0; deleted_at = null; }; language = null; word_count = null; format = null; } })'
r=$(pc "$USER_ID" memories_create "(\"$CAPSULE\", opt blob \"hello\", null, null, null, null, null, null, $META, \"idem-$RANDOM\")")
expect "the owner stores a memory" "$r" "Ok ="
MEMORY=$(echo "$r" | grep -oE '"[0-9a-f-]{36}"' | head -1 | tr -d '"')
r=$(pc "$OTHER_ID" memories_read "(\"$MEMORY\")")
expect "another user cannot read it" "$r" "NotFound"

# Upgrade, pulled by the owner: fetch the release from the factory, check its
# hash, install it with the owner's own identity.
TMP=$(mktemp -d)
dfx canister call canister_factory release_wasm --output raw 2>/dev/null | xxd -r -p > "$TMP/raw.bin"
python3 - "$TMP/raw.bin" "$TMP/release.wasm.gz" <<'PY'
import sys
b = open(sys.argv[1], 'rb').read()
i = b.find(b'\x1f\x8b')            # the gzip stream inside the Candid reply
open(sys.argv[2], 'wb').write(b[i:])
PY
[[ "$(sha256sum "$TMP/release.wasm.gz" | cut -d' ' -f1)" == "$RELEASE_SHA" ]] || fail "downloaded release does not match release_info"
echo_success "the owner downloads the release and its hash matches"
r=$(dfx canister install --identity "$USER_ID" "$PC" --mode upgrade --wasm "$TMP/release.wasm.gz" --yes 2>&1 | q)
expect "the owner upgrades their own canister" "$r" "Upgraded code"
r=$(dfx canister install "$PC" --mode upgrade --wasm "$TMP/release.wasm.gz" --yes 2>&1 || true)
expect "nobody else can upgrade it" "$r" "nly.*controllers"
rm -rf "$TMP"

r=$(pc "$USER_ID" memories_read "(\"$MEMORY\")")
expect "the memory is still there after the upgrade" "$r" "Ok ="
r=$(pc "$USER_ID" canister_mode)
expect "and it is still this user's personal canister" "$r" "owner = principal \"$USER_P\""
r=$(pc "$OTHER_ID" capsules_create '(null)')
expect "and still closed to others" "$r" "Unauthorized"

echo_success "✅ All personal canister tests passed ($PC)"
