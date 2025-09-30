pub type CapsuleId = String;

/// Pagination order for listing operations
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PaginationOrder {
    /// Ascending order (default)
    #[default]
    Asc,
}

/// Pagination result containing items
#[derive(Debug, Clone)]
pub struct Page<T> {
    /// The items for this page
    pub items: Vec<T>,
}

// Error types moved to main types.rs - use crate::types::Error instead
