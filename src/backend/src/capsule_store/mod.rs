//! Capsule Storage Foundation - Phase 1: Frozen API Surface
//!
//! This module defines the frozen CapsuleStore API that endpoints use.
//! Backends (Hash, Stable) implement this without exposing iterators.
//!
//! Frozen decisions:
//! - Enum-backed architecture (Store::{Hash,Stable}) — no trait objects
//! - Subject → ID = 1:1 (sparse multimap fallback if 1:N ever needed)
//! - Exclusive cursors for keyset pagination (id > after when Asc)
//! - Validation happens after closure; errors returned as UpdateError::Validation
//! - Hash backend may scan/sort for pagination (test-only); Stable uses ordered map

/// Frozen type alias for capsule identifiers throughout the system
pub type CapsuleId = String;

/// Pagination order for listing operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Order {
    /// Ascending order (default)
    Asc,
    /// Descending order
    Desc,
}

impl Default for Order {
    fn default() -> Self {
        Order::Asc
    }
}

/// Pagination result containing items and optional cursor for next page
#[derive(Debug, Clone)]
pub struct Page<T> {
    /// The items for this page
    pub items: Vec<T>,
    /// Cursor for the next page (None if no more items)
    pub next_cursor: Option<CapsuleId>,
}

/// Error types for storage operations
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateError {
    /// Item not found
    NotFound,
    /// Validation failed with message
    Validation(String),
    /// Concurrency conflict (placeholder for future MVCC)
    Concurrency,
}

/// Error type for put_if_absent operations
#[derive(Debug, Clone, PartialEq)]
pub enum AlreadyExists {
    /// Capsule with this ID already exists
    CapsuleExists(CapsuleId),
}

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
    ) -> Result<(), AlreadyExists>;

    /// Update a capsule with a closure (read-modify-write pattern)
    ///
    /// This is the primary mutation method that:
    /// - Handles index maintenance automatically
    /// - Provides atomic updates
    /// - Validation happens after the closure inside the store
    ///
    /// Returns Ok(()) if successful, or UpdateError if the capsule wasn't found
    /// or validation failed.
    fn update<F>(&mut self, id: &CapsuleId, f: F) -> Result<(), UpdateError>
    where
        F: FnOnce(&mut crate::types::Capsule);

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

    /// Paginate with default ascending order
    fn paginate_default(
        &self,
        after: Option<CapsuleId>,
        limit: u32,
    ) -> Page<crate::types::Capsule> {
        self.paginate(after, limit, Order::Asc)
    }

    /// Get total count of capsules (for metrics and pagination metadata)
    fn count(&self) -> u64;
}

// Include the backend implementations
pub mod hash;
pub mod stable;
pub mod store;

#[cfg(test)]
mod integration_tests;

// Re-export the main types for convenience
pub use crate::capsule_store::store::Store;
