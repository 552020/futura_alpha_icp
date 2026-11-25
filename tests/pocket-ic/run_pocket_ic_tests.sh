#!/bin/bash
# Build the canisters and run the PocketIC tests.
#
#   tests/pocket-ic/run_pocket_ic_tests.sh            # all three binaries
#   tests/pocket-ic/run_pocket_ic_tests.sh top_up     # one of: backend, memories, top_up
#
# PocketIC runs its own server process; a dfx replica is not touched.
# The crate needs server 10.0.0 and downloads it on first use. To skip the
# download, point POCKET_IC_BIN at that binary.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."

cargo build --target wasm32-unknown-unknown --release -p backend -p canister_factory

cd tests/pocket-ic
for name in ${1:-backend memories top_up}; do
    # One at a time: every test starts its own PocketIC instance
    cargo test --bin "${name}_pocket_ic" -- --test-threads=1
done
