pub mod adapters;
pub mod core;
pub mod types;

// Re-export the main functions for easy access
pub use adapters::{ping, CanisterEnv, StoreAdapter};
