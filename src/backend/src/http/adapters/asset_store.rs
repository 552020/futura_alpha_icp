use crate::http::core_types::{AssetStore, InlineAsset};

pub struct FuturaAssetStore;

impl AssetStore for FuturaAssetStore {
    fn get_inline(&self, memory_id: &str, asset_id: &str) -> Option<InlineAsset> {
        // 🔄 TODO: call your existing adapters to get small inline bytes
        // Example: memories_read_core(memory_id, asset_id) for small assets
        let _ = (memory_id, asset_id);
        None
    }
    
    fn get_blob_len(&self, memory_id: &str, asset_id: &str) -> Option<(u64, String)> {
        // 🔄 TODO: query your blob store for total length + content-type
        // Example: blob_store::get_metadata(memory_id, asset_id)
        let _ = (memory_id, asset_id);
        None
    }
    
    fn read_blob_chunk(&self, memory_id: &str, asset_id: &str, offset: u64, len: u64) -> Option<Vec<u8>> {
        // 🔄 TODO: stream a chunk from blob store
        // Example: blob_store::read_chunk(memory_id, asset_id, offset, len)
        let _ = (memory_id, asset_id, offset, len);
        None
    }
}
