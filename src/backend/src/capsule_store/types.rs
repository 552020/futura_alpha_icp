pub type CapsuleId = String;

/// Pagination order for listing operations
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PaginationOrder {
    /// Ascending order (default)
    #[default]
    Asc,
    /// Descending order
    Desc,
}

/// Pagination result containing items and optional cursor for next page
#[derive(Debug, Clone)]
pub struct Page<T> {
    /// The items for this page
    pub items: Vec<T>,
    /// Cursor for the next page (None if no more items)
    pub next_cursor: Option<CapsuleId>,
}

// Error types moved to main types.rs - use crate::types::Error instead
