#!/usr/bin/env bash
set -euo pipefail

# Config
BACKEND_CANISTER="${BACKEND_CANISTER:-backend}"
CAPSULE_ID="${CAPSULE_ID:?set CAPSULE_ID (principal string)}"

call() { dfx canister call "$BACKEND_CANISTER" "$1" "$2"; }

echo "== happy path =="
# meta is an empty record here; adjust to your MemoryMeta candid
SID=$(call uploads_begin "(principal \"$CAPSULE_ID\", record {}, 4:nat32, \"idem-1\")" \
  | awk -F'[()]' '/\(/ {print $2}' | tr -d ' ')

test -n "$SID" && echo "OK: session=$SID" || { echo "FAIL: no session returned"; exit 1; }

echo "== idempotency (same idem returns same sid) =="
SID2=$(call uploads_begin "(principal \"$CAPSULE_ID\", record {}, 4:nat32, \"idem-1\")" \
  | awk -F'[()]' '/\(/ {print $2}' | tr -d ' ')
test "$SID" = "$SID2" && echo "OK" || { echo "FAIL: idem did not return same sid"; exit 1; }

echo "== rejects zero chunks =="
set +e
OUT=$(call uploads_begin "(principal \"$CAPSULE_ID\", record {}, 0:nat32, \"idem-zero\")" 2>&1)
set -e
echo "$OUT" | grep -qi "expected_chunks_zero" && echo "OK" || { echo "FAIL: zero not rejected"; exit 1; }

# Optional: unauthorized check if you have another identity configured
if dfx identity list | grep -q other; then
  echo "== unauthorized principal =="
  dfx identity use other
  set +e
  OUT=$(call uploads_begin "(principal \"$CAPSULE_ID\", record {}, 4:nat32, \"idem-unauth\")" 2>&1)
  set -e
  dfx identity use default
  echo "$OUT" | grep -qi "Unauthorized" && echo "OK" || echo "WARN: Unauthorized not observed (check auth)"
fi

echo "uploads_begin smoke passed."
