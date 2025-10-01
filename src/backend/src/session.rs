// This file acts as the public interface for the session module.
// It re-exports key components from its sub-modules.

pub mod types;
pub mod service;
pub mod adapter;
pub mod compat;

// Re-export the main functions for easy access
pub use types::{SessionId, SessionSpec, SessionMeta, SessionStatus, ByteSink, Clock};
pub use service::SessionService;
pub use adapter::{SessionAdapter, ChunkIterator};
pub use compat::{SessionCompat, UploadSessionMeta};
