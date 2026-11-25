# Canister factory

Gives each user who wants one a canister of their own, running the same
backend code as the shared canister, and keeps it supplied with cycles. The
model is Juno's: the factory creates the canister, installs a fixed release,
hands it over and can no longer touch it.

Code: `src/lib.rs` (settings, admins, wasm upload), `src/personal.rs`
(releases, credits, creation, registry), `src/top_up.rs` (cycles).

## How a personal canister comes to be

1. **An admin releases the backend wasm.** `scripts/factory-release.sh <version> [network]`
   builds the backend, uploads it in chunks and calls `release_set`. New
   canisters always get this release; nobody else can upload code.
2. **An admin gives a user a credit:** `personal_credit_set(principal, 1)`. The
   web app's server does this after sign-up or payment. Without a credit there
   is no canister: creating one costs the factory real cycles.
3. **The user calls `create_personal_canister`.** The factory creates a
   canister with itself and the user as controllers, installs the release with
   the user as owner, then removes itself. The user is the only controller.
   The credit is used up when the canister exists. A failed step is resumed by
   calling again; no second canister is created.
4. **Upgrades are pulled by the owner:** download `release_wasm`, compare with
   `release_info().sha256_hex`, install with their own identity. The factory
   cannot push code.

```bash
dfx canister call canister_factory personal_credit_set '(principal "<user>", 1)'
dfx canister call canister_factory create_personal_canister        # as the user
dfx canister call canister_factory my_personal_canister            # as the user
dfx canister call canister_factory personal_canisters_list         # admin: all of them
```

On a local test network creation can be opened to everybody with
`personal_open_set '(true)'`. Never on mainnet.

## Cycles top-up

After the hand-off the factory is not a controller of a personal canister. It
can still keep it alive: the canister reports its cycles to the factory that
installed it, and depositing cycles needs no rights.

- Every hour a timer looks whether the last check is a day old; if so, the
  factory asks each `Ready` canister for `cycles_liquid_balance` (the cycles it
  can spend before it freezes).
- Below the floor (default 1T) it deposits up to the target (default 2T).
- Per owner it sends at most the monthly cap (default 3T per calendar month,
  UTC), and it never goes below its own `min_factory_cycles`.
- A frozen canister cannot answer; if it reported before, it counts as empty
  and is topped up.
- A canister whose owner installed other code can report anything. It still
  gets no more than its monthly cap.

```bash
# Settings (admin). enabled = false stops the timer; the calls below still work
dfx canister call canister_factory top_up_config
dfx canister call canister_factory top_up_config_set \
  '(record { enabled = true; floor = 1_000_000_000_000; target = 2_000_000_000_000; monthly_cap = 3_000_000_000_000 })'

# Another cap for one owner (admin); null = default, 0 = no top-ups
dfx canister call canister_factory top_up_cap_set '(principal "<owner>", opt 10_000_000_000_000)'

# Check now: one owner (admin, or the owner for themselves), or all (admin)
dfx canister call canister_factory top_up_check '(principal "<owner>")'
dfx canister call canister_factory top_up_check_all

# What happened: last balance, last top-up, this month's total, last error
dfx canister call canister_factory personal_canisters_list
```

Personal canisters installed before the backend remembered its factory do not
answer the factory. Their owner names it once:

```bash
dfx canister call <personal canister> personal_factory_set '(opt principal "<factory>")'
```

## Settings

Install argument (all optional):

| Field | Default | Meaning |
|---|---|---|
| `admins` | the deployer | Who may release, give credits and change settings |
| `min_factory_cycles` | 5T | The factory creates and tops up only while it holds more than this |
| `allowlist` | none | If set, only these principals may call `create_personal_canister` (they still need a credit) |
| `max_upload_size` | 50 MB | Largest wasm an admin can upload |
| `upload_ttl_seconds` | 24 h | An upload not turned into a release is dropped after this |

Later, by an admin: `add_admin`, `remove_admin`, `set_allowlist`,
`set_emergency_stop` (refuses every creation), `set_personal_canister_cycles`
(what a new canister starts with, default 2T), `cleanup_expired_uploads_manual`.

Open queries: `get_config`, `get_factory_stats`, `health_check` (says whether the
balance is above the minimum), `version`, `release_info`.

## What spends the factory's cycles

Only two things, both set up by an admin: creating a canister for a principal
with a credit, and topping up a canister in the registry within its cap.
There used to be a generic endpoint (`create_and_install_with`) that created a
canister for any caller with as many cycles as the caller named; it was
removed in November 2026.

Keep the factory funded: at 2T per new canister and up to 3T per owner per
month, a hundred owners can cost 300T a month in the worst case.

## Tests

```bash
cargo test -p canister_factory                      # the decisions, saved-state compatibility
tests/pocket-ic/run_pocket_ic_tests.sh top_up       # factory and personal canister on PocketIC
tests/backend/personal/                             # shell tests against a local replica
```

`canister_factory.did` is written by `scripts/check-did.sh --fix` from the
code's own export; do not edit it by hand.
