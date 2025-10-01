use crate::types::{AssetMetadata, CapsuleId, MemoryId};
use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{storable::Bound, Storable};
use std::borrow::Cow;

// Size constants aligned with senior developer feedback
pub const INLINE_MAX: u64 = 32 * 1024; // 32KB (fits in Capsule bound)
pub const CHUNK_SIZE: usize = 1_800_000; // 1.8MB - ICP expert recommended optimal size
                                         // Removed unused constant: PAGE_SIZE
pub const CAPSULE_INLINE_BUDGET: u64 = 32 * 1024; // Max inline bytes per capsule

/// Session identifier using u64 for efficient storage
#[derive(Clone, Debug, CandidType, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(pub u64);

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionId {
    pub fn new() -> Self {
        use crate::upload::sessions::STABLE_SESSION_COUNTER;
        let id = STABLE_SESSION_COUNTER.with(|counter| {
            let mut c = counter.borrow_mut();
            let current = c.get();
            let next_id = current + 1;
            c.set(next_id).expect("Failed to increment session counter");
            next_id
        });
        SessionId(id)
    }
}

impl Storable for SessionId {
    const BOUND: Bound = Bound::Bounded {
        max_size: 8,
        is_fixed_size: true,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(self.0.to_le_bytes().to_vec())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let arr: [u8; 8] = bytes
            .as_ref()
            .try_into()
            .expect("Invalid SessionId bytes - expected 8 bytes");
        SessionId(u64::from_le_bytes(arr))
    }
}

/// Blob identifier using u64 for efficient storage
#[derive(Clone, Debug, CandidType, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlobId(pub u64);

impl Default for BlobId {
    fn default() -> Self {
        Self::new()
    }
}

impl BlobId {
    pub fn new() -> Self {
        use crate::upload::blob_store::STABLE_BLOB_COUNTER;
        let id = STABLE_BLOB_COUNTER.with(|counter| {
            let mut c = counter.borrow_mut();
            let current = c.get();
            let next_id = current + 1;
            c.set(next_id).expect("Failed to increment blob counter");
            next_id
        });
        BlobId(id)
    }
}

impl Storable for BlobId {
    const BOUND: Bound = Bound::Bounded {
        max_size: 8,
        is_fixed_size: true,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(self.0.to_le_bytes().to_vec())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let arr: [u8; 8] = bytes
            .as_ref()
            .try_into()
            .expect("Invalid BlobId bytes - expected 8 bytes");
        BlobId(u64::from_le_bytes(arr))
    }
}

/// Session status for crash-safe commit workflow
#[derive(Clone, Debug, CandidType, Deserialize, PartialEq)]
pub enum SessionStatus {
    Pending,
    Committed { blob_id: u64 },
}

/// Upload session metadata with crash safety and authorization
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct SessionMeta {
    pub capsule_id: CapsuleId,
    pub provisional_memory_id: MemoryId,
    pub caller: candid::Principal,
    pub chunk_count: u32,
    pub expected_len: Option<u64>,
    pub expected_hash: Option<[u8; 32]>,
    pub status: SessionStatus,
    pub created_at: u64,
    pub asset_metadata: AssetMetadata,
    pub idem: String,
}

impl Storable for SessionMeta {
    const BOUND: Bound = Bound::Bounded {
        max_size: 2048,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(&(1u16, self)).expect("Failed to encode SessionMeta"))
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let (version, meta): (u16, SessionMeta) =
            Decode!(bytes.as_ref(), (u16, SessionMeta)).expect("Failed to decode SessionMeta");
        assert_eq!(version, 1, "Unsupported SessionMeta version");
        meta
    }
}

/// Blob metadata for integrity verification
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct BlobMeta {
    pub size: u64,
    pub checksum: [u8; 32],
    pub created_at: u64,
}

impl Storable for BlobMeta {
    const BOUND: Bound = Bound::Bounded {
        max_size: 512,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(&(1u16, self)).expect("Failed to encode BlobMeta"))
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let (version, meta): (u16, BlobMeta) =
            Decode!(bytes.as_ref(), (u16, BlobMeta)).expect("Failed to decode BlobMeta");
        assert_eq!(version, 1, "Unsupported BlobMeta version");
        meta
    }
}
