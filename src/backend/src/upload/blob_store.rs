use crate::memory::{MEM_BLOBS, MEM_BLOB_COUNTER, MEM_BLOB_META, MM};
use crate::types::Error;
use crate::upload::sessions::SessionStore;
use crate::upload::types::{BlobId, BlobMeta};
use hex;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::{StableBTreeMap, StableCell};
use sha2::{Digest, Sha256};
use std::cell::RefCell;

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static STABLE_BLOB_STORE: RefCell<StableBTreeMap<(u64, u32), Vec<u8>, Memory>> = RefCell::new(
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

    /// Store inline bytes and return blob reference
    pub fn put_inline(&self, bytes: &[u8]) -> Result<crate::types::BlobRef, Error> {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let sha256: [u8; 32] = hasher.finalize().into();

        let blob_id = BlobId::new();
        let store_key = format!("inline_{}", hex::encode(&sha256[..8]));

        // Store the bytes directly (for inline, we can store as a single page)
        let page_key = (blob_id.0, 0u32);
        STABLE_BLOB_STORE.with(|store| {
            store.borrow_mut().insert(page_key, bytes.to_vec());
        });

        // Store blob metadata
        let meta = BlobMeta {
            size: bytes.len() as u64,
            checksum: sha256,
            created_at: ic_cdk::api::time(),
        };

        STABLE_BLOB_META.with(|metastore| {
            metastore.borrow_mut().insert(blob_id.0, meta);
        });

        Ok(crate::types::BlobRef {
            locator: store_key,
            hash: Some(sha256),
            len: bytes.len() as u64,
        })
    }

    /// Store chunks from session as a blob with integrity verification
    pub fn store_from_chunks(
        &self,
        session_store: &SessionStore,
        session_id: &crate::upload::types::SessionId,
        chunk_count: u32,
        expected_len: u64,
        expected_hash: [u8; 32],
    ) -> Result<BlobId, Error> {
        let blob_id = BlobId::new();
        let mut hasher = Sha256::new();
        let mut total_written = 0u64;

        // Stream chunks into blob store pages
        let chunk_iter = session_store.iter_chunks(session_id, chunk_count);
        for (page_idx, chunk_data) in chunk_iter.enumerate() {
            // Debug logging: Log the exact bytes being hashed
            let first_10_bytes = if chunk_data.len() >= 10 {
                format!("{:?}", &chunk_data[..10])
            } else {
                format!("{:?}", &chunk_data[..])
            };
            ic_cdk::println!(
                "STORE_FROM_CHUNKS: session_id={}, page_idx={}, data_len={}, first_10_bytes={}",
                session_id.0,
                page_idx,
                chunk_data.len(),
                first_10_bytes
            );

            hasher.update(&chunk_data);
            total_written += chunk_data.len() as u64;

            // Store as blob page
            let page_key = (blob_id.0, page_idx as u32);
            STABLE_BLOB_STORE.with(|store| {
                store.borrow_mut().insert(page_key, chunk_data);
            });
        }

        // Verify integrity
        let actual_hash: [u8; 32] = hasher.finalize().into();
        if actual_hash != expected_hash {
            // Cleanup on failure
            self.delete_blob(&blob_id)?;
            return Err(Error::InvalidArgument(format!(
                "checksum_mismatch: expected={}, actual={}",
                hex::encode(expected_hash),
                hex::encode(actual_hash)
            )));
        }
        if total_written != expected_len {
            // Cleanup on failure
            self.delete_blob(&blob_id)?;
            return Err(Error::InvalidArgument(format!(
                "size_mismatch: expected={}, actual={}",
                expected_len, total_written
            )));
        }

        // Store blob metadata
        let meta = BlobMeta {
            size: total_written,
            checksum: actual_hash,
            created_at: ic_cdk::api::time(),
        };

        STABLE_BLOB_META.with(|metas| {
            metas.borrow_mut().insert(blob_id.0, meta);
        });

        Ok(blob_id)
    }

    /// Read entire blob (use carefully - can be large)
    pub fn read_blob(&self, blob_id: &BlobId) -> Result<Vec<u8>, Error> {
        let meta = STABLE_BLOB_META
            .with(|metas| metas.borrow().get(&blob_id.0))
            .ok_or(Error::NotFound)?;

        let mut result = Vec::with_capacity(meta.size as usize);
        let mut page_idx = 0u32;

        loop {
            let page_key = (blob_id.0, page_idx);
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
    pub fn get_blob_meta(&self, blob_id: &BlobId) -> Result<Option<BlobMeta>, Error> {
        let meta = STABLE_BLOB_META.with(|metas| metas.borrow().get(&blob_id.0));
        Ok(meta)
    }

    /// Delete blob and all its pages
    pub fn delete_blob(&self, blob_id: &BlobId) -> Result<(), Error> {
        // Delete metadata first
        STABLE_BLOB_META.with(|metas| metas.borrow_mut().remove(&blob_id.0));

        // Delete all pages
        let mut page_idx = 0u32;
        loop {
            let page_key = (blob_id.0, page_idx);
            let removed = STABLE_BLOB_STORE.with(|store| store.borrow_mut().remove(&page_key));

            if removed.is_none() {
                break; // No more pages
            }
            page_idx += 1;
        }

        Ok(())
    }

    /// Check if blob exists
    pub fn blob_exists(&self, blob_id: &BlobId) -> bool {
        STABLE_BLOB_META.with(|metas| metas.borrow().contains_key(&blob_id.0))
    }

    /// Get blob metadata by store key (for verification)
    pub fn head(&self, store_key: &str) -> Result<Option<BlobMeta>, Error> {
        // Parse store_key to extract hash prefix
        // Format: "inline_{hex_hash_prefix}" or other locator formats
        let hash_prefix = if store_key.starts_with("inline_") {
            store_key.strip_prefix("inline_")
        } else {
            // For other locator types, we might need different parsing
            // For now, assume inline format or return None
            None
        };

        if let Some(prefix_hex) = hash_prefix {
            // Try to decode the hex prefix
            if let Ok(prefix_bytes) = hex::decode(prefix_hex) {
                // Search through all blob metadata for a matching hash prefix
                let result = STABLE_BLOB_META.with(|metas| {
                    for (_blob_id, meta) in metas.borrow().iter() {
                        // Check if the hash starts with our prefix
                        if meta.checksum.len() >= prefix_bytes.len()
                            && meta.checksum[..prefix_bytes.len()] == prefix_bytes[..]
                        {
                            return Some(meta.clone());
                        }
                    }
                    None
                });
                Ok(result)
            } else {
                Ok(None) // Invalid hex in store_key
            }
        } else {
            Ok(None) // Unsupported store_key format
        }
    }

    /// Get total number of blobs (for monitoring)
    pub fn blob_count(&self) -> u64 {
        STABLE_BLOB_META.with(|metas| metas.borrow().len())
    }

    /// Get total storage used by blobs (for monitoring)
    pub fn total_storage_used(&self) -> u64 {
        STABLE_BLOB_META.with(|metas| metas.borrow().iter().map(|(_, meta)| meta.size).sum())
    }
}

/// Read blob data by locator (public API function)
/// Automatically chooses between single response and chunked reading based on size
pub fn blob_read(locator: String) -> std::result::Result<Vec<u8>, Error> {
    use crate::upload::types::BlobId;
    use hex;

    // Parse locator to extract blob ID
    // Format: "inline_{hex_hash_prefix}" or "blob_{blob_id}"
    let blob_id = if locator.starts_with("inline_") {
        // For inline blobs, we need to find the blob by hash prefix
        let hash_prefix = locator.strip_prefix("inline_").unwrap_or("");
        if let Ok(prefix_bytes) = hex::decode(hash_prefix) {
            // Search through blob metadata to find matching blob
            let blob_store = BlobStore::new();
            let mut found_blob_id = None;

            // We need to iterate through all blobs to find one with matching hash prefix
            // This is not ideal for performance, but necessary for the current architecture
            for blob_id_num in 0..blob_store.blob_count() {
                let blob_id = BlobId(blob_id_num);
                if let Ok(Some(meta)) = blob_store.get_blob_meta(&blob_id) {
                    // Compare the first 8 bytes of the full checksum with the prefix
                    if meta.checksum.len() >= prefix_bytes.len()
                        && meta.checksum[..prefix_bytes.len()] == prefix_bytes[..]
                    {
                        found_blob_id = Some(blob_id);
                        break;
                    }
                }
            }

            found_blob_id.ok_or(crate::types::Error::NotFound)?
        } else {
            return Err(crate::types::Error::InvalidArgument(
                "Invalid hex in locator".to_string(),
            ));
        }
    } else if locator.starts_with("blob_") {
        // For blob_ format, extract the numeric ID
        let id_str = locator.strip_prefix("blob_").unwrap_or("");
        let blob_id_num: u64 = id_str.parse().map_err(|_| {
            crate::types::Error::InvalidArgument("Invalid blob ID in locator".to_string())
        })?;
        BlobId(blob_id_num)
    } else {
        return Err(crate::types::Error::InvalidArgument(
            "Unsupported locator format".to_string(),
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
    const CHUNK_SIZE: u32 = 1024 * 1024; // 1MB chunks
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
    blob_store: &BlobStore,
    blob_id: &crate::upload::types::BlobId,
    chunk_index: u32,
) -> std::result::Result<Vec<u8>, Error> {
    // Note: The stable blob store is accessed directly via STABLE_BLOB_STORE

    let page_key = (blob_id.0, chunk_index);
    let page_data = STABLE_BLOB_STORE.with(|store| store.borrow().get(&page_key));

    Ok(page_data.unwrap_or_default())
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
            let page_key = (blob_id.0, chunk_index);
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
    fn test_blob_read_inline_locator() {
        let _blob_store = create_test_blob_store();

        // Test reading blob by inline hash prefix
        let result = blob_read("inline_0102030405".to_string());
        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data, b"Hello, World! This is test data for blob reading.");
    }

    #[test]
    fn test_blob_read_inline_locator_invalid_hex() {
        // Test with invalid hex in inline locator
        let result = blob_read("inline_invalid_hex".to_string());
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::types::Error::InvalidArgument(_) => {}
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_read_blob_chunked_large_blob() {
        let blob_store = BlobStore::new();

        // Create a large blob that exceeds MAX_SINGLE_RESPONSE_SIZE
        let large_data = vec![0u8; 3 * 1024 * 1024]; // 3MB
        let blob_id = BlobId(1);

        // Store the blob data in chunks
        let chunk_size = 1024 * 1024; // 1MB chunks
        let mut chunk_index = 0;
        for chunk in large_data.chunks(chunk_size) {
            let page_key = (blob_id.0, chunk_index);
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
