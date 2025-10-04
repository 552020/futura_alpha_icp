//! Core memory management operations
//!
//! This module contains the core business logic for memory management,
//! organized by operation type for better maintainability.

pub(crate) mod model_helpers;
pub mod traits;

pub mod assets;
pub mod create;
pub mod delete;
pub mod read;
pub mod update;

// Re-export the public core API so callers use `memories::core::*`
pub use assets::{
    asset_remove_core, asset_remove_external_core, asset_remove_inline_core,
    asset_remove_internal_core, memories_cleanup_assets_all_core,
    memories_cleanup_assets_bulk_core, memories_list_assets_core,
};
pub use create::memories_create_core;
pub use delete::{memories_delete_all_core, memories_delete_bulk_core, memories_delete_core};
pub use read::memories_read_core;
pub use traits::{Env, Store};
pub use update::memories_update_core;
