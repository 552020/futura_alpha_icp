#!/bin/bash
# Inheritance end to end on the deployed backend: an owner shares a memory
# "after death", a trustee confirms the death, the heir can read it only then,
# and the owner can cancel. Uses three dfx identities.
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../test_utils.sh"

OWNER=default
for id in futura-test-trustee futura-test-heir; do
    dfx identity list 2>/dev/null | grep -qx "$id" || dfx identity new "$id" --storage-mode plaintext >/dev/null 2>&1
done
TRUSTEE=$(dfx identity get-principal --identity futura-test-trustee)
HEIR=$(dfx identity get-principal --identity futura-test-heir)

call() { local who="$1"; shift; dfx canister call --identity "$who" backend "$@" 2>&1 | grep -v "^WARNING"; }
fail() { echo_error "$1"; exit 1; }
expect() { echo "$2" | tr -s '\n ' ' ' | grep -q "$3" || fail "$1: expected '$3', got: $2"; echo_success "$1"; }

CAPSULE=$(call $OWNER capsules_read_basic '(null)' | grep -o 'capsule_id = "[^"]*"' | head -1 | cut -d'"' -f2)
[[ -z "$CAPSULE" ]] && CAPSULE=$(call $OWNER capsules_create '(null)' | grep -o 'id = "capsule_[^"]*"' | head -1 | cut -d'"' -f2)
[[ -z "$CAPSULE" ]] && fail "no capsule"

MEMORY=$(create_test_memory "$CAPSULE" "for_my_heir" "opens after my death" '"legacy";' 'blob "Zm9yIHlvdQ=="')
[[ -z "$MEMORY" ]] && fail "could not create the memory"
OWNER_P=$(dfx identity get-principal --identity $OWNER)
NOW=$(date +%s)000000000
entry() { # principal role condition
    echo "record { id = \"e-$2\"; person_ref = opt variant { Principal = principal \"$1\" }; is_public = false; grant_source = variant { User }; source_id = null; role = variant { $2 }; perm_mask = 3 : nat32; invited_by_person_ref = null; created_at = $NOW : nat64; updated_at = $NOW : nat64; condition = $3 }"
}
OWNER_ENTRY=$(entry "$OWNER_P" Owner 'variant { Immediate }' | sed 's/perm_mask = 3/perm_mask = 31/')
HEIR_ENTRY=$(entry "$HEIR" Guest 'variant { EventTriggered = record { event = variant { AfterDeath } } }')
r=$(call $OWNER memories_update "(\"$MEMORY\", record { name = null; metadata = null; access_entries = opt vec { $OWNER_ENTRY; $HEIR_ENTRY } })")
expect "owner shares the memory with the heir, after death" "$r" "Ok ="

# start clean: confirmations of an earlier run would still count
call $OWNER legacy_cancel_death "(\"$CAPSULE\")" >/dev/null

# trustee: 1 of 1, waiting period 0 seconds so the test can see it take effect
r=$(call $OWNER legacy_settings_set "(\"$CAPSULE\", vec { principal \"$TRUSTEE\" }, 1 : nat32, 0 : nat64)")
expect "owner names a trustee" "$r" "trustees = vec { principal \"$TRUSTEE\""

r=$(call futura-test-heir memories_read_shared "(\"$CAPSULE\", \"$MEMORY\")")
expect "heir reads nothing before the death" "$r" "NotFound"
r=$(call futura-test-heir legacy_settings_set "(\"$CAPSULE\", vec { principal \"$HEIR\" }, 1 : nat32, 0 : nat64)")
expect "heir cannot make themselves trustee" "$r" "NotFound"
r=$(call futura-test-heir legacy_confirm_death "(\"$CAPSULE\")")
expect "heir cannot confirm the death" "$r" "NotFound"
r=$(call futura-test-heir memories_read "(\"$MEMORY\")")
expect "heir cannot use the owner's read endpoint" "$r" "NotFound"

r=$(call futura-test-trustee legacy_confirm_death "(\"$CAPSULE\")")
expect "trustee confirms the death" "$r" "death_in_effect = true"

r=$(call futura-test-heir memories_list_shared "(\"$CAPSULE\", null, null)")
expect "heir lists the memory" "$r" "$MEMORY"
r=$(call futura-test-heir memories_read_shared "(\"$CAPSULE\", \"$MEMORY\")")
expect "heir reads the memory" "$r" "Ok ="
echo "$r" | grep -q 'id = "e-Owner"' && fail "the heir can see the owner's access entry"
echo "$r" | grep -q 'id = "e-Guest"' || fail "the heir does not see their own access entry"
echo_success "heir sees only their own access entry"
r=$(call futura-test-trustee memories_read_shared "(\"$CAPSULE\", \"$MEMORY\")")
expect "trustee confirmed but can read nothing" "$r" "NotFound"
r=$(call futura-test-trustee legacy_withdraw_confirmation "(\"$CAPSULE\")")
expect "trustee cannot undo a death in effect" "$r" "Conflict"

r=$(call $OWNER legacy_cancel_death "(\"$CAPSULE\")")
expect "owner cancels: alive" "$r" "death_in_effect = false"
r=$(call futura-test-heir memories_read_shared "(\"$CAPSULE\", \"$MEMORY\")")
expect "heir is locked out again" "$r" "NotFound"

r=$(call $OWNER legacy_event_record "(\"$CAPSULE\", variant { AfterDeath })")
expect "a death cannot be recorded as a plain event" "$r" "InvalidArgument"

# leave the capsule without trustees
call $OWNER legacy_settings_set "(\"$CAPSULE\", vec {}, 1 : nat32, 2592000 : nat64)" >/dev/null
echo_success "✅ All inheritance tests passed"
