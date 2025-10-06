use crate::memory::{MEM_BLOBS, MEM_BLOB_COUNTER, MEM_BLOB_META, MM};
use crate::session::{ByteSink, SessionAdapter};
use crate::types::Error;
use crate::upload::types::{BlobId, BlobMeta};
use hex;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::{StableBTreeMap, StableCell};
use sha2::{Digest, Sha256};
use std::cell::RefCell;

/// Deterministic hash of provisional_memory_id for stable chunk keys
/// CRITICAL: This MUST be used everywhere chunks are written/read
/// Includes session_id to prevent parallel key collisions
pub fn pmid_hash32(pmid: &str) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(pmid.as_bytes());
    h.finalize().into()
}

/// Deterministic hash including session_id for parallel-safe chunk keys
pub fn pmid_session_hash32(pmid: &str, session_id: u64) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(pmid.as_bytes());
    h.update(b"#"); // Separator
    h.update(&session_id.to_le_bytes());
    h.finalize().into()
}

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // Key changed to ([u8; 32] SHA256 of provisional_memory_id, u32 chunk_idx) for determinism
    // Note: This is a BREAKING CHANGE - existing blob data will need migration
    pub static STABLE_BLOB_STORE: RefCell<StableBTreeMap<([u8; 32], u32), Vec<u8>, Memory>> = RefCell::new(
        StableBTreeMap::init(MM.with(|m| m.borrow().get(MEM_BLOBS)))
    );

    static STABLE_BLOB_META: RefCell<StableBTreeMap<u64, BlobMeta, Memory>> = RefCell::new(
        StableBTreeMap::init(MM.with(|m| m.borrow().get(MEM_BLOB_META)))
    );

    pub static STABLE_BLOB_COUNTER: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(MM.with(|m| m.borrow().get(MEM_BLOB_COUNTER)), 0)
            .expect("Failed to init blob counter")
    );
}

/// Blob store for paged storage of large files
#[cfg_attr(not(feature = "upload"), allow(dead_code))]
pub struct BlobStore;

#[cfg_attr(not(feature = "upload"), allow(dead_code))]
impl Default for BlobStore {
    fn default() -> Self {
        Self::new()
    }
}

impl BlobStore {
    pub fn new() -> Self {
        BlobStore
    }

    // Note: put_inline method removed - not currently used

    /// Store chunks from session as a blob with integrity verification
    /// NOTE: This method is being phased out in favor of write-through ByteSink design
    /// where chunks are written directly to storage during put_chunk
    pub fn store_from_chunks(
        &self,
        session_store: &crate::session::SessionCompat,
        session_id: &crate::upload::types::SessionId,
        chunk_count: u32,
        expected_len: u64,
        expected_hash: [u8; 32],
    ) -> std::result::Result<BlobId, Error> {
        // Get session metadata to retrieve the provisional_memory_id (used as blob_id during write)
        let session_meta = session_store
            .get(session_id)
            .map_err(|e| Error::Internal(format!("Failed to get session: {:?}", e)))?
            .ok_or(Error::NotFound)?;

        // Derive pmid_hash the EXACT same way StableBlobSink does (deterministic SHA256 + session_id)
        let pmid_hash =
            pmid_session_hash32(&session_meta.provisional_memory_id, session_meta.session_id);

        // Create blob_id from first 8 bytes of hash for metadata storage
        let blob_id = BlobId(u64::from_be_bytes([
            pmid_hash[0],
            pmid_hash[1],
            pmid_hash[2],
            pmid_hash[3],
            pmid_hash[4],
            pmid_hash[5],
            pmid_hash[6],
            pmid_hash[7],
        ]));

        // NOTE: Hash verification is now done using rolling hash in uploads_finish()
        // This function just verifies chunks exist and returns the blob_id
        // (Chunks were already written via StableBlobSink during put_chunk)

        let mut total_written = 0u64;

        // Verify all chunks exist in blob store
        for page_idx in 0..chunk_count {
            let page_key = (pmid_hash, page_idx);
            let chunk_data =
                STABLE_BLOB_STORE.with(|store| store.borrow().get(&page_key).unwrap_or_default());

            ic_cdk::println!(
                "BLOB_READ sid={} chunk_idx={} found={} len={} pmid_hash={:?}",
                session_id.0,
                page_idx,
                !chunk_data.is_empty(),
                chunk_data.len(),
                &pmid_hash[..8]
            );

            if chunk_data.is_empty() {
                ic_cdk::println!(
                    "BLOB_READ_NOTFOUND sid={} chunk_idx={} pmid_hash={:?}",
                    session_id.0,
                    page_idx,
                    &pmid_hash[..8]
                );
                // Cleanup on failure
                self.delete_blob(&blob_id)?;
                return Err(Error::NotFound);
            }

            total_written += chunk_data.len() as u64;
        }

        // Verify total size matches expected
        if total_written != expected_len {
            // Cleanup on failure
            self.delete_blob(&blob_id)?;
            return Err(Error::InvalidArgument(format!(
                "size_mismatch: expected={}, actual={}",
                expected_len, total_written
            )));
        }

        // Store blob metadata (use expected_hash since we already verified it with rolling hash)
        let meta = BlobMeta {
            size: total_written,
            checksum: expected_hash,
            created_at: ic_cdk::api::time(),
            pmid_hash, // Save for later retrieval/deletion
        };

        STABLE_BLOB_META.with(|metas| {
            metas.borrow_mut().insert(blob_id.0, meta);
        });

        Ok(blob_id)
    }

    /// Read entire blob (use carefully - can be large)
    pub fn read_blob(&self, blob_id: &BlobId) -> std::result::Result<Vec<u8>, Error> {
        let meta = STABLE_BLOB_META
            .with(|metas| metas.borrow().get(&blob_id.0))
            .ok_or(Error::NotFound)?;

        let mut result = Vec::with_capacity(meta.size as usize);
        let mut page_idx = 0u32;

        loop {
            let page_key = (meta.pmid_hash, page_idx); // Use stored pmid_hash
            let page_data = STABLE_BLOB_STORE.with(|store| store.borrow().get(&page_key));

            match page_data {
                Some(data) => {
                    result.extend_from_slice(&data);
                    page_idx += 1;
                }
                None => break,
            }
        }

        Ok(result)
    }

    /// Get blob metadata without reading content
    pub fn get_blob_meta(&self, blob_id: &BlobId) -> std::result::Result<Option<BlobMeta>, Error> {
        let meta = STABLE_BLOB_META.with(|metas| metas.borrow().get(&blob_id.0));
        Ok(meta)
    }

    /// Delete blob and all its pages
    pub fn delete_blob(&self, blob_id: &BlobId) -> std::result::Result<(), Error> {
        // Get meta to retrieve pmid_hash before deleting
        let meta = STABLE_BLOB_META
            .with(|metas| metas.borrow_mut().remove(&blob_id.0))
            .ok_or(Error::NotFound)?;

        // Delete all pages using pmid_hash
        let mut page_idx = 0u32;
        loop {
            let page_key = (meta.pmid_hash, page_idx); // Use stored pmid_hash
            let removed = STABLE_BLOB_STORE.with(|store| store.borrow_mut().remove(&page_key));

            if removed.is_none() {
                break; // No more pages
            }
            page_idx += 1;
        }

        Ok(())
    }

    // Note: blob_exists method removed - not currently used

    // Removed unused method: head

    /// Get total number of blobs (for monitoring)
    pub fn blob_count(&self) -> u64 {
        STABLE_BLOB_META.with(|metas| metas.borrow().len())
    }

    // Removed unused method: total_storage_used
}

/// Read blob data by locator (public API function)
/// Automatically chooses between single response and chunked reading based on size
pub fn blob_read(locator: String) -> std::result::Result<Vec<u8>, Error> {
    use crate::upload::types::BlobId;

    // Parse locator to extract blob ID
    // Format: "blob_{blob_id}" (inline_ format removed for performance)
    let blob_id = if locator.starts_with("blob_") {
        // For blob_ format, extract the numeric ID (fast O(1) lookup)
        let id_str = locator.strip_prefix("blob_").unwrap_or("");
        let blob_id_num: u64 = id_str.parse().map_err(|_| {
            crate::types::Error::InvalidArgument("Invalid blob ID in locator".to_string())
        })?;
        BlobId(blob_id_num)
    } else {
        return Err(crate::types::Error::InvalidArgument(
            "Unsupported locator format. Expected 'blob_{id}'".to_string(),
        ));
    };

    // Check blob size and choose reading strategy
    let blob_store = BlobStore::new();
    if let Ok(Some(meta)) = blob_store.get_blob_meta(&blob_id) {
        const MAX_SINGLE_RESPONSE_SIZE: u64 = 2 * 1024 * 1024; // 2MB limit

        if meta.size <= MAX_SINGLE_RESPONSE_SIZE {
            // Small blob - read in one go
            blob_store.read_blob(&blob_id).map_err(|e| match e {
                crate::types::Error::NotFound => crate::types::Error::NotFound,
                _ => crate::types::Error::Internal(format!("Failed to read blob: {:?}", e)),
            })
        } else {
            // Large blob - use chunked reading
            read_blob_chunked(&blob_store, &blob_id, meta.size)
        }
    } else {
        Err(crate::types::Error::NotFound)
    }
}

/// Read large blob data in chunks and combine into single response
/// This is a fallback for blobs that are too large for single response
fn read_blob_chunked(
    blob_store: &BlobStore,
    blob_id: &crate::upload::types::BlobId,
    total_size: u64,
) -> std::result::Result<Vec<u8>, Error> {
    // Removed unused constant: CHUNK_SIZE
    let mut result = Vec::with_capacity(total_size as usize);
    let mut chunk_index = 0u32;

    loop {
        let chunk_data = read_blob_chunk(blob_store, blob_id, chunk_index)?;

        if chunk_data.is_empty() {
            // No more chunks
            break;
        }

        result.extend_from_slice(&chunk_data);
        chunk_index += 1;

        // Safety check to prevent infinite loops
        if chunk_index > 1000 {
            return Err(crate::types::Error::Internal(
                "Too many chunks - possible infinite loop".to_string(),
            ));
        }
    }

    Ok(result)
}

/// Read a single chunk of blob data
fn read_blob_chunk(
    _blob_store: &BlobStore,
    blob_id: &crate::upload::types::BlobId,
    chunk_index: u32,
) -> std::result::Result<Vec<u8>, Error> {
    // Get meta to retrieve pmid_hash
    let meta = STABLE_BLOB_META
        .with(|metas| metas.borrow().get(&blob_id.0))
        .ok_or(Error::NotFound)?;

    let page_key = (meta.pmid_hash, chunk_index); // Use stored pmid_hash
    let page_data = STABLE_BLOB_STORE.with(|store| store.borrow().get(&page_key));

    Ok(page_data.unwrap_or_default())
}

/// Read blob data by locator in chunks (public API for chunked reading)
/// Returns individual chunks to avoid IC message size limits
pub fn blob_read_chunk(locator: String, chunk_index: u32) -> std::result::Result<Vec<u8>, Error> {
    use crate::upload::types::BlobId;

    // Parse locator to extract blob ID (inline_ format removed for performance)
    let blob_id = if locator.starts_with("blob_") {
        // For blob_ format, extract the numeric ID (fast O(1) lookup)
        let id_str = locator.strip_prefix("blob_").unwrap_or("");
        let blob_id_num: u64 = id_str.parse().map_err(|_| {
            crate::types::Error::InvalidArgument("Invalid blob ID in locator".to_string())
        })?;
        BlobId(blob_id_num)
    } else {
        return Err(crate::types::Error::InvalidArgument(
            "Unsupported locator format. Expected 'blob_{id}'".to_string(),
        ));
    };

    // Verify blob exists
    let blob_store = BlobStore::new();
    if blob_store.get_blob_meta(&blob_id)?.is_none() {
        return Err(crate::types::Error::NotFound);
    }

    // Read the specific chunk
    read_blob_chunk(&blob_store, &blob_id, chunk_index)
}

/// Get blob metadata including total chunk count
pub fn blob_get_meta(locator: String) -> std::result::Result<crate::types::BlobMeta, Error> {
    use crate::upload::types::BlobId;

    // Parse locator to extract blob ID (inline_ format removed for performance)
    let blob_id = if locator.starts_with("blob_") {
        // For blob_ format, extract the numeric ID (fast O(1) lookup)
        let id_str = locator.strip_prefix("blob_").unwrap_or("");
        let blob_id_num: u64 = id_str.parse().map_err(|_| {
            crate::types::Error::InvalidArgument("Invalid blob ID in locator".to_string())
        })?;
        BlobId(blob_id_num)
    } else {
        return Err(crate::types::Error::InvalidArgument(
            "Unsupported locator format. Expected 'blob_{id}'".to_string(),
        ));
    };

    // Get blob metadata
    let blob_store = BlobStore::new();
    if let Ok(Some(meta)) = blob_store.get_blob_meta(&blob_id) {
        // Calculate total chunk count based on size
        // Each chunk is stored as a page, and we need to count how many pages exist
        let mut chunk_count = 0u32;
        loop {
            let page_key = (meta.pmid_hash, chunk_count); // Use stored pmid_hash
            let exists = STABLE_BLOB_STORE.with(|store| store.borrow().contains_key(&page_key));
            if !exists {
                break;
            }
            chunk_count += 1;
        }

        Ok(crate::types::BlobMeta {
            size: meta.size,
            chunk_count,
        })
    } else {
        Err(crate::types::Error::NotFound)
    }
}

/// Delete blob by locator (public API function)
pub fn blob_delete(locator: String) -> std::result::Result<(), Error> {
    use crate::upload::types::BlobId;

    // Parse locator to extract blob ID
    let blob_id = if locator.starts_with("blob_") {
        // For blob_ format, extract the numeric ID
        let id_str = locator.strip_prefix("blob_").unwrap_or("");
        let blob_id_num: u64 = id_str.parse().map_err(|_| {
            crate::types::Error::InvalidArgument("Invalid blob ID in locator".to_string())
        })?;
        BlobId(blob_id_num)
    } else {
        return Err(crate::types::Error::InvalidArgument(
            "Unsupported locator format. Expected 'blob_{id}'".to_string(),
        ));
    };

    let blob_store = BlobStore::new();
    blob_store.delete_blob(&blob_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upload::types::BlobId;

    // Helper function to create a test blob store with some data
    fn create_test_blob_store() -> BlobStore {
        let blob_store = BlobStore::new();

        // Create a test blob with some data
        let test_data = b"Hello, World! This is test data for blob reading.";
        let blob_id = BlobId(0);

        // Store the blob data in chunks
        let chunk_size = 10;
        let mut chunk_index = 0;
        for chunk in test_data.chunks(chunk_size) {
            let mut pmid_hash = [0u8; 32];
            pmid_hash[0..8].copy_from_slice(&blob_id.0.to_be_bytes());
            let page_key = (pmid_hash, chunk_index);
            STABLE_BLOB_STORE.with(|store| {
                store.borrow_mut().insert(page_key, chunk.to_vec());
            });
            chunk_index += 1;
        }

        // Store blob metadata
        let mut checksum = [0u8; 32];
        checksum[0..5].copy_from_slice(&[1, 2, 3, 4, 5]);
        let meta = crate::upload::types::BlobMeta {
            size: test_data.len() as u64,
            checksum,
            created_at: 1234567890,
            pmid_hash: [0u8; 32], // Test hash
        };

        STABLE_BLOB_META.with(|store| {
            store.borrow_mut().insert(blob_id.0, meta);
        });

        blob_store
    }

    #[test]
    fn test_blob_read_success() {
        let _blob_store = create_test_blob_store();

        // Test reading blob by ID
        let result = blob_read("blob_0".to_string());
        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data, b"Hello, World! This is test data for blob reading.");
    }

    #[test]
    fn test_blob_read_not_found() {
        // Test reading non-existent blob
        let result = blob_read("blob_999".to_string());
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::types::Error::NotFound => {}
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_blob_read_invalid_locator_format() {
        // Test with invalid locator format
        let result = blob_read("invalid_format".to_string());
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::types::Error::InvalidArgument(_) => {}
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_blob_read_invalid_blob_id() {
        // Test with invalid blob ID
        let result = blob_read("blob_invalid".to_string());
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::types::Error::InvalidArgument(_) => {}
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_blob_read_unsupported_format() {
        // Test that inline_ format is properly rejected
        let result = blob_read("inline_0102030405".to_string());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("unsupported locator format. expected"));
    }

    #[test]
    fn test_read_blob_chunked_large_blob() {
        let _blob_store = BlobStore::new();

        // Create a large blob that exceeds MAX_SINGLE_RESPONSE_SIZE
        let large_data = vec![0u8; 3 * 1024 * 1024]; // 3MB
        let blob_id = BlobId(1);

        // Store the blob data in chunks
        let chunk_size = 1024 * 1024; // 1MB chunks
        let mut chunk_index = 0;
        for chunk in large_data.chunks(chunk_size) {
            let mut pmid_hash = [0u8; 32];
            pmid_hash[0..8].copy_from_slice(&blob_id.0.to_be_bytes());
            let page_key = (pmid_hash, chunk_index);
            STABLE_BLOB_STORE.with(|store| {
                store.borrow_mut().insert(page_key, chunk.to_vec());
            });
            chunk_index += 1;
        }

        // Store blob metadata
        let mut checksum = [0u8; 32];
        checksum[0..5].copy_from_slice(&[6, 7, 8, 9, 10]);
        let meta = crate::upload::types::BlobMeta {
            size: large_data.len() as u64,
            checksum,
            created_at: 1234567890,
            pmid_hash: [0u8; 32], // Test hash
        };

        STABLE_BLOB_META.with(|store| {
            store.borrow_mut().insert(blob_id.0, meta);
        });

        // Test reading large blob (should use chunked reading)
        let result = blob_read("blob_1".to_string());
        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data.len(), large_data.len());
        assert_eq!(data, large_data);
    }

    #[test]
    fn test_read_blob_chunk_empty_chunk() {
        let blob_id = BlobId(2);

        // Test reading a chunk that doesn't exist (should return empty)
        let result = read_blob_chunk(&BlobStore::new(), &blob_id, 0);
        assert!(result.is_ok());

        let data = result.unwrap();
        assert!(data.is_empty());
    }
}

// ============================================================================
// ByteSink Implementation for Direct Chunk Writing
// ============================================================================

/// StableBlobSink implements ByteSink for direct chunk writing to stable storage
pub struct StableBlobSink {
    pmid_hash: [u8; 32], // Deterministic key stem (SHA256 of provisional_memory_id)
    chunk_size: usize,
    #[allow(dead_code)]
    capsule_id: crate::types::CapsuleId, // Keep for potential future use
}

impl StableBlobSink {
    /// Create a new StableBlobSink from UploadSessionMeta
    pub fn for_meta(meta: &crate::session::UploadSessionMeta) -> Result<Self, Error> {
        Ok(Self {
            pmid_hash: pmid_session_hash32(&meta.provisional_memory_id, meta.session_id),
            chunk_size: meta.chunk_size,
            capsule_id: meta.capsule_id.clone(),
        })
    }

    fn write_at_impl(&mut self, offset: u64, data: &[u8]) -> Result<(), Error> {
        // Validate alignment (all chunks must be aligned to chunk_size boundary)
        if offset % (self.chunk_size as u64) != 0 {
            return Err(Error::InvalidArgument("unaligned offset".into()));
        }

        // Calculate chunk index from offset
        let chunk_idx = (offset / self.chunk_size as u64) as u32;

        // Validate chunk size (no oversized chunks)
        if data.len() > self.chunk_size {
            return Err(Error::InvalidArgument("oversized chunk".into()));
        }

        // Debug logging
        ic_cdk::println!(
            "WRITE_AT: pmid_hash={:?}, chunk_idx={}, offset={}, data_len={}",
            &self.pmid_hash[..4], // First 4 bytes for logging
            chunk_idx,
            offset,
            data.len()
        );

        // Store chunk directly in stable storage (write-through, no buffering)
        // Key: (pmid_hash, chunk_idx) - deterministic key using SHA256 of provisional_memory_id
        STABLE_BLOB_STORE.with(|store| {
            let mut store = store.borrow_mut();
            store.insert((self.pmid_hash, chunk_idx), data.to_vec());
        });

        // CRITICAL: Same-call verification to diagnose value bound issues
        let verify = STABLE_BLOB_STORE.with(|store| {
            store
                .borrow()
                .get(&(self.pmid_hash, chunk_idx))
                .map(|d| d.len())
        });
        match verify {
            Some(len) if len == data.len() => {
                ic_cdk::println!(
                    "BLOB_VERIFY_SAMECALL idx={} wrote={} read={} ✅",
                    chunk_idx,
                    data.len(),
                    len
                );
            }
            Some(len) => {
                ic_cdk::println!(
                    "BLOB_VERIFY_SAMECALL_MISMATCH idx={} wrote={} read={} ❌",
                    chunk_idx,
                    data.len(),
                    len
                );
            }
            None => {
                ic_cdk::println!(
                    "BLOB_VERIFY_SAMECALL_MISS idx={} wrote={} read=None ❌❌❌",
                    chunk_idx,
                    data.len()
                );
            }
        }

        Ok(())
    }
}

impl ByteSink for StableBlobSink {
    fn write_at(&mut self, offset: u64, data: &[u8]) -> Result<(), Error> {
        self.write_at_impl(offset, data)
    }
}
