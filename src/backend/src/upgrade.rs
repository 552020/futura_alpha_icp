//! What survives an upgrade, and how.
//!
//! Everything lives in stable structures behind the one memory manager
//! (memory.rs) and persists by itself. `pre_upgrade` must not call
//! `ic_cdk::storage::stable_save`: that writes a raw snapshot at offset 0 of
//! stable memory, on top of the memory manager's header and first buckets.
//! Older versions of this canister did exactly that. The capsules came back
//! (they were in the snapshot), but files in the blob store, upload sessions
//! and every other stable structure were lost on each upgrade.
//!
//! Upgrading FROM such a version: during the upgrade the OLD code's
//! `pre_upgrade` runs one last time and destroys the files again. The
//! capsules come back (`legacy_snapshot_present` sees the snapshot and
//! `post_upgrade` restores it), the files do not. So upgrade such a canister
//! with the old hook skipped, which leaves stable memory untouched:
//!
//!     dfx canister install backend --mode upgrade --skip-pre-upgrade --wasm <new wasm>
//!
//! The memory layout (ids 0 to 9 and 42) is the same in both versions, so the
//! new code finds capsules and files as they were. Done this way on mainnet
//! on 25 November 2026, after a plain upgrade had lost the files and the
//! snapshot taken before it was loaded back.

const MANAGER_MAGIC: &[u8; 3] = b"MGR";

/// True when stable memory holds a snapshot written by an old `pre_upgrade`
/// instead of the memory manager's header. Call it first in `post_upgrade`,
/// before anything touches a stable structure.
pub fn legacy_snapshot_present() -> bool {
    if ic_cdk::stable::stable_size() == 0 {
        return false;
    }
    let mut magic = [0u8; 3];
    ic_cdk::stable::stable_read(0, &mut magic);
    &magic != MANAGER_MAGIC
}
