#!/bin/bash
# Cycles top-up of personal canisters: the canister tells the factory what it
# can still spend, the factory deposits when that is below the floor, up to a
# monthly cap per owner.
# Needs the local replica with canister_factory deployed and a release that
# has cycles_liquid_balance (scripts/factory-release.sh <version>).
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$SCRIPT_DIR/../../.."
source "$SCRIPT_DIR/../test_utils.sh"
cd "$ROOT"

fail() { echo_error "$1"; exit 1; }
expect() { echo "$2" | tr -s '\n ' ' ' | grep -q "$3" || fail "$1: expected '$3', got: $2"; echo_success "$1"; }
q() { grep -v "^WARNING"; }
f() { dfx canister call canister_factory "$@" 2>&1 | q; }
num() { grep -oE '[0-9_]+ : nat' | head -1 | tr -d '_ ' | cut -d: -f1; }

USER_ID="futura-test-topup-$RANDOM"
OTHER_ID=futura-test-heir
dfx identity new "$USER_ID" --storage-mode plaintext >/dev/null 2>&1
dfx identity list 2>/dev/null | grep -qx "$OTHER_ID" || dfx identity new "$OTHER_ID" --storage-mode plaintext >/dev/null 2>&1
USER_P=$(dfx identity get-principal --identity "$USER_ID")
DEFAULT_CFG='(record { enabled = true; floor = 1_000_000_000_000; target = 2_000_000_000_000; monthly_cap = 3_000_000_000_000 })'
trap 'f top_up_config_set "$DEFAULT_CFG" >/dev/null || true; dfx identity remove "$USER_ID" >/dev/null 2>&1 || true' EXIT

dfx ledger fabricate-cycles --canister canister_factory --t 20 >/dev/null 2>&1 || true
f top_up_config_set "$DEFAULT_CFG" >/dev/null
f personal_credit_set "(principal \"$USER_P\", 1)" >/dev/null
r=$(dfx canister call --identity "$USER_ID" canister_factory create_personal_canister 2>&1 | q)
expect "the user gets a personal canister" "$r" "status = variant { Ready }"
PC=$(echo "$r" | grep -oE 'canister_id = opt principal "[^"]+"' | cut -d'"' -f2)
pc() { local who="$1"; shift; dfx canister call --identity "$who" --candid src/backend/backend.did "$PC" "$@" 2>&1 | q; }

r=$(pc "$USER_ID" cycles_liquid_balance)
BEFORE=$(echo "$r" | num)
[[ -n "$BEFORE" && "$BEFORE" -gt 0 ]] || fail "the owner cannot read the balance (old release? run scripts/factory-release.sh): $r"
echo_success "the owner reads what the canister can still spend ($BEFORE cycles)"
r=$(pc "$OTHER_ID" cycles_liquid_balance)
expect "nobody else reads it" "$r" "owner and the factory only"

r=$(dfx canister call --identity "$OTHER_ID" canister_factory top_up_check "(principal \"$USER_P\")" 2>&1 | q)
expect "a stranger cannot trigger a top-up for somebody else" "$r" "Admin access required"

r=$(f top_up_check "(principal \"$USER_P\")")
expect "a fresh canister has enough: nothing is sent" "$r" 'note = "enough"'
expect "and the factory read the balance from the canister" "$r" "balance = opt"

# Raise the floor above what the canister holds; cap the month at 1T
f top_up_config_set '(record { enabled = true; floor = 5_000_000_000_000; target = 6_000_000_000_000; monthly_cap = 1_000_000_000_000 })' >/dev/null
r=$(dfx canister call --identity "$USER_ID" canister_factory top_up_check "(principal \"$USER_P\")" 2>&1 | q)
expect "below the floor the factory deposits, up to the monthly cap" "$r" "sent = 1_000_000_000_000"
AFTER=$(pc "$USER_ID" cycles_liquid_balance | num)
GAIN=$((AFTER - BEFORE))
[[ "$GAIN" -gt 900000000000 && "$GAIN" -le 1000000000000 ]] || fail "the canister did not receive the cycles: before $BEFORE, after $AFTER"
echo_success "the cycles arrived in the canister (+$GAIN)"

r=$(f top_up_check "(principal \"$USER_P\")")
expect "the cap is reached: a second check sends nothing" "$r" "monthly cap reached"
expect "nothing was sent" "$r" "sent = 0"

r=$(f top_up_cap_set "(principal \"$USER_P\", opt 1_500_000_000_000)")
expect "an admin raises this owner's cap" "$r" "Ok"
r=$(f top_up_check "(principal \"$USER_P\")")
expect "and the rest up to the new cap is sent" "$r" "sent = 500_000_000_000"

r=$(dfx canister call --identity "$USER_ID" canister_factory my_personal_canister 2>&1 | q)
expect "the registry records the month's total" "$r" "month_total = opt (1_500_000_000_000"

r=$(dfx canister call --identity "$USER_ID" canister_factory top_up_config_set "$DEFAULT_CFG" 2>&1 | q)
expect "only admins change the settings" "$r" "Admin access required"

echo_success "✅ All top-up tests passed ($PC)"
