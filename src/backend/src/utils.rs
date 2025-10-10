// ✅ MODERN RUST: Main utils module file
pub mod blob_id;
pub mod name_conversion;  // ✅ ADD: New name conversion utilities
pub mod uuid_v7;

// Re-export specific items for convenience
pub use name_conversion::*;  // ✅ ADD: Export name conversion functions
pub use uuid_v7::*;
