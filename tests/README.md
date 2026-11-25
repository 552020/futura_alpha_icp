# Tests

Three layers, from fast to slow. State of 25 November 2026.

| Layer | Where | Needs | Run | Count |
|---|---|---|---|---|
| Unit tests | next to the code, `src/backend`, `src/canister_factory` | nothing | `cargo test -p backend -p canister_factory` | 149 + 19 |
| PocketIC tests | `tests/pocket-ic/` | the built wasms | `tests/pocket-ic/run_pocket_ic_tests.sh` | 12 |
| Shell tests | `tests/backend/` | a local dfx replica with the canisters deployed | `tests/backend/run_all_tests.sh` | 22 |

## Unit tests

Pure logic: access rules, capsule store, uploads, galleries, inheritance,
tokens, the top-up decision. No network.

## PocketIC tests

The real wasms on a local Internet Computer in one process: calls between
canisters, timers, upgrades (also from the old backend), cycles, HTTP with
signed links. See `tests/pocket-ic/README.md`.

## Shell tests

Scripts that call a deployed canister with dfx. The runner works from
`tests/backend/test_registry.sh`: one row per test, numbered.

```bash
cd tests/backend
./run_all_tests.sh --list          # the 22 tests
./run_all_tests.sh                 # run them all
./run_all_tests.sh --start-from 9  # or from a number
./test_status.sh                   # what passed in the last run
```

To add a test, write the script and append a row to the registry.

## Not part of any run

`tests/backend` holds many more files than the 22 registered tests: 41 further
shell scripts and about 100 `.mjs` scripts (uploads, HTTP, bulk memory
calls, mainnet helpers, `utils/`). They were written during development, are
run by hand if at all, and nobody has checked recently which of them still
work. Treat them as notes until one is fixed and registered.
