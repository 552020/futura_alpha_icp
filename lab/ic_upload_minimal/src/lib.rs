//! Minimal Internet Computer Upload Flow
//!
//! This crate demonstrates the core concepts of a chunked upload
//! system without the complexity of the full implementation.

use candid::{CandidType, Deserialize};
use std::collections::HashMap;

/// Session ID for tracking uploads
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SessionId(pub u64);

/// Upload session state
#[derive(CandidType, Deserialize, Debug)]
pub struct UploadSession {
    pub session_id: SessionId,
    pub total_chunks: u32,
    pub received_chunks: HashMap<u32, Vec<u8>>,
    pub is_complete: bool,
}

/// Minimal upload service
pub struct MinimalUploadService {
    sessions: HashMap<SessionId, UploadSession>,
}

impl MinimalUploadService {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Start a new upload session
    pub fn begin_upload(&mut self, session_id: SessionId, total_chunks: u32) {
        let session = UploadSession {
            session_id: session_id.clone(),
            total_chunks,
            received_chunks: HashMap::new(),
            is_complete: false,
        };
        self.sessions.insert(session_id, session);
    }

    /// Put a chunk of data
    pub fn put_chunk(&mut self, session_id: &SessionId, chunk_index: u32, data: Vec<u8>) -> Result<(), String> {
        let session = self.sessions.get_mut(session_id)
            .ok_or("Session not found")?;

        if chunk_index >= session.total_chunks {
            return Err("Chunk index out of bounds".to_string());
        }

        if data.len() > 1024 * 1024 { // 1MB limit
            return Err("Chunk too large".to_string());
        }

        session.received_chunks.insert(chunk_index, data);

        // Check if upload is complete
        if session.received_chunks.len() == session.total_chunks as usize {
            session.is_complete = true;
        }

        Ok(())
    }

    /// Commit the upload and get all data
    pub fn commit_upload(&mut self, session_id: &SessionId) -> Result<Vec<u8>, String> {
        let session = self.sessions.get_mut(session_id)
            .ok_or("Session not found")?;

        if !session.is_complete {
            return Err("Upload not complete".to_string());
        }

        // Combine all chunks in order
        let mut result = Vec::new();
        for i in 0..session.total_chunks {
            if let Some(chunk) = session.received_chunks.get(&i) {
                result.extend_from_slice(chunk);
            } else {
                return Err(format!("Missing chunk {}", i));
            }
        }

        // Clean up session
        self.sessions.remove(session_id);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_upload_flow() {
        let mut service = MinimalUploadService::new();
        let session_id = SessionId(123);

        // Begin upload
        service.begin_upload(session_id.clone(), 3);

        // Upload chunks
        service.put_chunk(&session_id, 0, b"Hel".to_vec()).unwrap();
        service.put_chunk(&session_id, 1, b"lo,".to_vec()).unwrap();
        service.put_chunk(&session_id, 2, b" world!".to_vec()).unwrap();

        // Commit and verify
        let result = service.commit_upload(&session_id).unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "Hello, world!");
    }

    #[test]
    fn test_chunk_validation() {
        let mut service = MinimalUploadService::new();
        let session_id = SessionId(456);

        service.begin_upload(session_id.clone(), 2);

        // Test oversized chunk
        let large_chunk = vec![0u8; 2 * 1024 * 1024]; // 2MB
        let result = service.put_chunk(&session_id, 0, large_chunk);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too large"));
    }
}
