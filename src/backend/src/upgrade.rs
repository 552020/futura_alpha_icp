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
//! Upgrading FROM such a version still works: its `pre_upgrade` leaves the
//! snapshot at offset 0, `legacy_snapshot_present` sees that the memory does
//! not start with the manager's magic, and `post_upgrade` restores from the
//! snapshot one last time. From then on the header stays intact.

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
