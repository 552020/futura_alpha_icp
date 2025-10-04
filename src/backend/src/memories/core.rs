//! Core memory management functions with dependency injection
//!
//! This module implements the decoupling pattern:
//! - Pure business logic separated from ICP-specific APIs
//! - Trait-based dependency injection for testability
//! - Post-write assertions to catch silent failures

pub mod traits;
pub mod model_helpers;
pub mod create;
pub mod read;
pub mod update;
pub mod delete;
pub mod assets;

// Re-export the public surface (clean API)
pub use traits::{Env, Store};
pub use create::memories_create_core;
pub use read::memories_read_core;
pub use update::memories_update_core;
pub use delete::{
    memories_delete_core,
    memories_delete_bulk_core,
    memories_delete_all_core,
};
pub use assets::{
    cleanup_memory_assets,
    cleanup_internal_blob_asset,
    cleanup_external_blob_asset,
    cleanup_vercel_blob_asset,
    cleanup_s3_blob_asset,
    cleanup_arweave_blob_asset,
    cleanup_ipfs_blob_asset,
    cleanup_neon_blob_asset,
    memories_cleanup_assets_all_core,
    memories_cleanup_assets_bulk_core,
    asset_remove_core,
    asset_remove_inline_core,
    asset_remove_internal_core,
    asset_remove_external_core,
    memories_list_assets_core,
};