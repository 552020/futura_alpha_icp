pub mod adapters;
pub mod core;
pub mod types;
pub mod utils;

// Re-export the main functions for easy access
pub use adapters::{ping, CanisterEnv, StoreAdapter};

// Re-export new asset link types for external use
pub use utils::{AssetKind, AssetLink, AssetLinks};
