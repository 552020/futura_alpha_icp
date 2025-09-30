//! Capsule Storage Foundation - Phase 1: Frozen API Surface
//!
//! This module defines the frozen CapsuleStore API that endpoints use.
//! Backends (Hash, Stable) implement this without exposing iterators.
//!
//! Frozen decisions:
//! - Enum-backed architecture (Store::{Hash,Stable}) — no trait objects
//! - Subject → ID = 1:1 (sparse multimap fallback if 1:N ever needed)
//! - Exclusive cursors for keyset pagination (id > after when Asc)
//! - Validation happens after closure; errors returned as Error::InvalidArgument
//! - Hash backend may scan/sort for pagination (test-only); Stable uses ordered map

// Import types from our types module
use crate::capsule_store::types::{CapsuleId, Page, PaginationOrder as Order};

/// FROZEN: The core storage trait that endpoints will use
///
/// This trait provides a clean abstraction over persistence that:
/// - Works with both HashMap (testing) and StableBTreeMap (production)
/// - Provides all necessary operations without exposing iterators
/// - Maintains clean separation between business logic and persistence
/// - Is future-proof for scaling and feature additions
pub trait CapsuleStore {
    /// Check if a capsule exists by ID
    fn exists(&self, id: &CapsuleId) -> bool;

    /// Get a capsule by ID
    fn get(&self, id: &CapsuleId) -> Option<crate::types::Capsule>;

    /// Put a capsule (insert or update), returning the previous value
    fn upsert(
        &mut self,
        id: CapsuleId,
        capsule: crate::types::Capsule,
    ) -> Option<crate::types::Capsule>;

    /// Put a capsule only if it doesn't already exist
    fn put_if_absent(
        &mut self,
        id: CapsuleId,
        capsule: crate::types::Capsule,
    ) -> Result<(), crate::types::Error>;

    /// Update a capsule with a closure (read-modify-write pattern)
    ///
    /// This is the primary mutation method that:
    /// - Handles index maintenance automatically
    /// - Provides atomic updates
    /// - Validation happens after the closure inside the store
    ///
    /// Returns Ok(()) if successful, or Error if the capsule wasn't found
    /// or validation failed.
    fn update<F>(&mut self, id: &CapsuleId, f: F) -> Result<(), crate::types::Error>
    where
        F: FnOnce(&mut crate::types::Capsule);

    /// Update a capsule with a closure that returns a result
    ///
    /// This method allows the closure to return a `std::result::Result<R, Error>`, enabling
    /// proper error propagation from within the update operation. This eliminates
    /// the need for silent early returns and provides better error handling.
    ///
    /// Returns the result from the closure, or Error if the capsule wasn't found.
    fn update_with<R, F>(&mut self, id: &CapsuleId, f: F) -> Result<R, crate::types::Error>
    where
        F: FnOnce(&mut crate::types::Capsule) -> Result<R, crate::types::Error>;

    /// Remove a capsule by ID
    fn remove(&mut self, id: &CapsuleId) -> Option<crate::types::Capsule>;

    /// Find capsule by subject (1:1 relationship - frozen decision)
    fn find_by_subject(&self, subject: &crate::types::PersonRef) -> Option<crate::types::Capsule>;

    /// List capsules by owner (returns IDs for pagination compatibility)
    fn list_by_owner(&self, owner: &crate::types::PersonRef) -> Vec<CapsuleId>;

    /// Get multiple capsules by IDs (batch operation)
    fn get_many(&self, ids: &[CapsuleId]) -> Vec<crate::types::Capsule>;

    /// Paginate capsules with keyset pagination
    ///
    /// Cursor semantics (FROZEN):
    /// - `after` is EXCLUSIVE: returns items with `id > after` when `Asc`, `id < after` when `Desc`
    /// - `order` defaults to `Asc` (ascending by CapsuleId)
    /// - `next_cursor` is the last item's ID for continuation
    ///
    /// This provides efficient, consistent pagination without O(n) scans.
    fn paginate(
        &self,
        after: Option<CapsuleId>,
        limit: u32,
        order: Order,
    ) -> Page<crate::types::Capsule>;

    // Removed unused method: paginate_default

    /// Get total count of capsules (for metrics and pagination metadata)
    fn count(&self) -> u64;

    /// Get storage statistics
    /// Returns (capsules_count, subject_index_count, owner_index_count)
    fn stats(&self) -> (u64, u64, u64);
}

// Include the backend implementations
// pub mod hash;  // LEGACY - commented out, we only use StableStore now
pub mod stable;
pub mod store;
pub mod types;

#[cfg(test)]
mod integration_tests;

// Re-export the main types for convenience
pub use crate::capsule_store::store::Store;
