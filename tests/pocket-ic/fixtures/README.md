# Fixtures

`backend-bee859d.wasm.gz`: the backend as of commit `bee859d` (20 November
2026), gzipped. It is the last version whose `pre_upgrade` wrote a snapshot
over the start of stable memory and whose signing key is derived from public
data. `backend_pocket_ic.rs` installs it, stores a file and upgrades to the
current code, to prove that a canister deployed with an old version (mainnet)
can be upgraded without losing data.

Rebuild:

```bash
git worktree add /tmp/old-backend bee859d
cd /tmp/old-backend
cargo build --target wasm32-unknown-unknown --release -p backend
gzip -9 -n -c target/wasm32-unknown-unknown/release/backend.wasm > backend-bee859d.wasm.gz
```
