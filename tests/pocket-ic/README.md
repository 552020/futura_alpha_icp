# PocketIC tests

Integration tests that run the real canister wasms on
[PocketIC](https://github.com/dfinity/pocketic), a local Internet Computer in
one process. They cover what unit tests cannot (calls between canisters,
timers, upgrades, cycles, HTTP) and need no dfx replica.

## Run

```bash
tests/pocket-ic/run_pocket_ic_tests.sh           # build the wasms, run everything
tests/pocket-ic/run_pocket_ic_tests.sh top_up    # one binary: backend, memories or top_up
```

The `pocket-ic` crate (10.0) needs PocketIC server 10.0.0. It downloads the
server on first use; set `POCKET_IC_BIN` to an existing binary to skip that.
The server bundled with dfx 0.31 is 11.0.0 and is refused.

## What is tested

| File | Tests | Covers |
|---|---|---|
| `backend_pocket_ic.rs` | 4 | The backend starts and answers; the signing key made at install is replaced by a random one, once, and kept across upgrades; a file is served only for a link the canister signed (no token, changed token, other file are refused); an upgrade from the old backend in `fixtures/` keeps the data |
| `memories_pocket_ic.rs` | 5 | Memories: create and read, update, delete, idempotent creation, a stranger cannot delete |
| `top_up_pocket_ic.rs` | 3 | Cycles top-up of personal canisters by the factory: who may read the cycles, floor and target, monthly cap and its reset in a new month, the hourly timer (also after a factory upgrade), a canister that stops reporting, rescue of a frozen canister, the factory's own minimum |

`common.rs` holds what the three share: loading a wasm and installing a backend.

## Writing a test

- Take reply and argument types from the `backend` crate (`use backend::types::*`).
  Copies of the candid types in a test go out of date with every interface change;
  four of these tests failed for months for that reason.
- Install the backend with `common::install_backend`, or pass `encode_args(())`
  as install argument. Empty bytes are not candid and make `init` trap.
- A test that only prints cannot fail. Assert.
