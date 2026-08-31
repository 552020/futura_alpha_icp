// This file acts as the public interface for the session module.
// It re-exports key components from its sub-modules.

pub mod adapter;
pub mod compat;
pub mod service;
pub mod types;

// Re-export the main functions for easy access
pub use compat::{SessionCompat, UploadSessionMeta};
pub use types::{ByteSink, SessionId};
