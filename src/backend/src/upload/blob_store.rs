use crate::memory_manager::{MEM_BLOBS, MEM_BLOB_COUNTER, MEM_BLOB_META, MM};
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
            kind: crate::types::MemoryBlobKind::ICPCapsule,
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
            return Err(Error::InvalidArgument("checksum".to_string()));
        }
        if total_written != expected_len {
            // Cleanup on failure
            self.delete_blob(&blob_id)?;
            return Err(Error::InvalidArgument("size".to_string()));
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
