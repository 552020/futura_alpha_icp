//! Core memory management functions with dependency injection
//!
//! This module implements the decoupling pattern:
//! - Pure business logic separated from ICP-specific APIs
//! - Trait-based dependency injection for testability
//! - Post-write assertions to catch silent failures

pub mod assets;
pub mod create;
pub mod delete;
pub mod model_helpers;
pub mod read;
pub mod traits;
pub mod update;

// Re-export the public surface (clean API)
pub use assets::{
    asset_get_by_id_core, asset_remove_by_id_core, asset_remove_core, asset_remove_external_core,
    asset_remove_inline_core, asset_remove_internal_core, memories_cleanup_assets_all_core,
    memories_cleanup_assets_bulk_core, memories_list_assets_core,
};
pub use create::memories_create_core;
pub use delete::{memories_delete_all_core, memories_delete_bulk_core, memories_delete_core};
pub use read::memories_read_core;
pub use traits::{Env, Store};
pub use update::{memories_update_core, memories_add_asset_core, memories_add_inline_asset_core};
