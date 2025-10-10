use crate::gallery::domain::Gallery;

/// Estimate the size of a gallery in bytes
pub fn estimate_gallery_size(gallery: &Gallery) -> u64 {
    // Estimate based on gallery metadata and memory entries
    let mut size = 0u64;

    // Gallery metadata size
    size += gallery.id.len() as u64;
    if let Some(title) = &gallery.metadata.title {
        size += title.len() as u64;
    }
    if let Some(description) = &gallery.metadata.description {
        size += description.len() as u64;
    }

    // Memory entries size
    for memory_entry in &gallery.items {
        size += memory_entry.memory_id.len() as u64;
        if let Some(caption) = &memory_entry.caption {
            size += caption.len() as u64;
        }
        size += memory_entry.metadata.len() as u64;
    }

    size
}

/// Estimate the size of a gallery within a capsule in bytes
pub fn estimate_gallery_capsule_size(gallery: &Gallery) -> u64 {
    // This is the same as estimate_gallery_size for now
    estimate_gallery_size(gallery)
}

/// Get a human-readable size report for a gallery
pub fn get_gallery_size_report(gallery: &Gallery) -> String {
    let size_bytes = estimate_gallery_size(gallery);
    let size_kb = size_bytes / 1024;
    let size_mb = size_kb / 1024;

    if size_mb > 0 {
        format!("{} MB ({} KB, {} bytes)", size_mb, size_kb, size_bytes)
    } else if size_kb > 0 {
        format!("{} KB ({} bytes)", size_kb, size_bytes)
    } else {
        format!("{} bytes", size_bytes)
    }
}

/// Get detailed size breakdown for a gallery
pub fn get_gallery_size_breakdown(gallery: &Gallery) -> GallerySizeInfo {
    let total_size = estimate_gallery_size(gallery);
    let memory_count = gallery.items.len() as u64;

    GallerySizeInfo {
        total_size_bytes: total_size,
        memory_count,
        average_memory_size: if memory_count > 0 {
            total_size / memory_count
        } else {
            0
        },
        metadata_size: {
            let mut size = 0u64;
            size += gallery.id.len() as u64;
            if let Some(title) = &gallery.metadata.title {
                size += title.len() as u64;
            }
            if let Some(description) = &gallery.metadata.description {
                size += description.len() as u64;
            }
            size
        },
    }
}

/// Gallery size information structure
#[derive(Debug, Clone, candid::CandidType, serde::Deserialize, serde::Serialize)]
pub struct GallerySizeInfo {
    pub total_size_bytes: u64,
    pub memory_count: u64,
    pub average_memory_size: u64,
    pub metadata_size: u64,
}
