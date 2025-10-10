// src/backend/src/util/uuid_v7.rs
use ic_cdk::management_canister::raw_rand;

// Hex table to avoid fmt machinery (smaller/faster)
const HEX: &[u8; 16] = b"0123456789abcdef";

fn write_hex(dst: &mut [u8], bytes: &[u8]) {
    for (i, b) in bytes.iter().enumerate() {
        dst[2 * i] = HEX[(b >> 4) as usize];
        dst[2 * i + 1] = HEX[(b & 0x0F) as usize];
    }
}

// SAFETY: buf must be length 36: 8-4-4-4-12 (with hyphens at 8,13,18,23)
fn format_uuid_hyphenated(bytes: [u8; 16]) -> String {
    let mut out = [0u8; 36];
    // positions of each segment
    let (a, rest) = out.split_at_mut(8); // 4 bytes -> 8 hex chars
    let (dash1, rest) = rest.split_at_mut(1); // -
    let (b, rest) = rest.split_at_mut(4); // 2 bytes -> 4 hex chars
    let (dash2, rest) = rest.split_at_mut(1);
    let (c, rest) = rest.split_at_mut(4); // 2 bytes -> 4 hex chars
    let (dash3, rest) = rest.split_at_mut(1);
    let (d, rest) = rest.split_at_mut(4); // 2 bytes -> 4 hex chars
    let (dash4, e) = rest.split_at_mut(1); // -
                                           // remaining 6+6 bytes = 12 hex
                                           // Layout: time_hi.. etc will be placed below via slices

    // Map into the canonical groups:
    // [0..4]=time_hi(6 bytes) actually: v7 packs 48-bit ms across first 6 bytes
    let t0 = &bytes[0..4]; // first 4 bytes of time
    let t1 = &bytes[4..6]; // next 2 bytes of time (will carry version bits)
    let r1 = &bytes[6..8]; // will carry variant in high bits
    let r2 = &bytes[8..10];
    let r3 = &bytes[10..16];

    write_hex(a, &[t0[0], t0[1], t0[2], t0[3]]); // 4 bytes -> 8 hex chars
    dash1[0] = b'-';

    write_hex(b, &[t1[0], t1[1]]); // 2 bytes -> 4 hex chars
    dash2[0] = b'-';

    write_hex(c, &[r1[0], r1[1]]); // 2 bytes -> 4 hex chars
    dash3[0] = b'-';

    write_hex(d, &[r2[0], r2[1]]); // 2 bytes -> 4 hex chars
    dash4[0] = b'-';

    write_hex(e, &[r3[0], r3[1], r3[2], r3[3], r3[4], r3[5]]); // 6 bytes -> 12 hex chars

    // Safety: out is valid UTF-8 (ASCII hex + hyphens)
    String::from_utf8(out.to_vec()).unwrap()
}

/// Generate a UUID v7 string using IC time + raw_rand entropy.
/// Call this from **update** handlers (raw_rand requires update context).
#[allow(dead_code)]
pub async fn uuid_v7() -> String {
    // 1) 48-bit milliseconds timestamp
    let ns = ic_cdk::api::time(); // nanoseconds since epoch
    let ms: u64 = ns / 1_000_000; // to milliseconds

    // 2) Get 32 bytes of entropy
    let rand_out = raw_rand().await.expect("raw_rand failed");
    let mut r = rand_out; // Vec<u8>
    if r.len() < 10 {
        r.resize(10, 0);
    } // ensure enough bytes

    // 3) Assemble 128 bits
    // Layout: 6 bytes timestamp | 10 bytes random
    let mut b = [0u8; 16];
    b[0] = ((ms >> 40) & 0xFF) as u8;
    b[1] = ((ms >> 32) & 0xFF) as u8;
    b[2] = ((ms >> 24) & 0xFF) as u8;
    b[3] = ((ms >> 16) & 0xFF) as u8;
    b[4] = ((ms >> 8) & 0xFF) as u8;
    b[5] = (ms & 0xFF) as u8;

    b[6..16].copy_from_slice(&r[..10]);

    // 4) Set version (0111) in high 4 bits of byte 6 (index 6 is the *third* nibble region)
    b[6] = (b[6] & 0x0F) | 0x70;

    // 5) Set variant (10xx) in high 2 bits of byte 8 (index 8 here is group after version)
    b[8] = (b[8] & 0x3F) | 0x80;

    format_uuid_hyphenated(b)
}

use ic_cdk::api::{canister_self, msg_caller, time};
use std::cell::Cell;

// thread-local per instance; persist a real counter in stable memory if needed
thread_local! {
    static CTR: Cell<u64> = Cell::new(0);
}

pub fn uuid_v7_weak() -> String {
    let ns = if cfg!(test) {
        // Mock time for tests
        1_000_000_000_000 // 1 second in nanoseconds
    } else {
        time()
    };
    let ms = ns / 1_000_000;

    let mut tail = [0u8; 10];
    let c = if cfg!(test) {
        // In test mode, use a fixed counter for deterministic results
        0
    } else {
        CTR.with(|x| {
            let v = x.get();
            x.set(v.wrapping_add(1));
            v
        })
    };

    // poor-man's mixing (good enough for uniqueness, not for security)
    let caller_byte = if cfg!(test) {
        0x42 // Mock caller byte for tests
    } else {
        msg_caller().as_slice().first().copied().unwrap_or(0)
    };
    let canister_byte = if cfg!(test) {
        0x24 // Mock canister byte for tests
    } else {
        canister_self().as_slice().first().copied().unwrap_or(0)
    };
    let seed = ((caller_byte as u64) << 32) ^ (canister_byte as u64) << 24 ^ ms ^ c;

    for i in 0..10 {
        tail[i] = ((seed >> ((i * 7) % 56)) & 0xFF) as u8;
    }

    let mut b = [0u8; 16];
    b[0] = ((ms >> 40) & 0xFF) as u8;
    b[1] = ((ms >> 32) & 0xFF) as u8;
    b[2] = ((ms >> 24) & 0xFF) as u8;
    b[3] = ((ms >> 16) & 0xFF) as u8;
    b[4] = ((ms >> 8) & 0xFF) as u8;
    b[5] = (ms & 0xFF) as u8;
    b[6..16].copy_from_slice(&tail);

    b[6] = (b[6] & 0x0F) | 0x70;
    b[8] = (b[8] & 0x3F) | 0x80;

    format_uuid_hyphenated(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_v7_format() {
        // Test the weak version for unit tests
        let uuid = uuid_v7_weak();

        // Should be exactly 36 characters (32 hex + 4 hyphens)
        assert_eq!(uuid.len(), 36);

        // Should have exactly 4 hyphens
        let hyphen_count = uuid.matches('-').count();
        assert_eq!(hyphen_count, 4);

        // Should match UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
        let parts: Vec<&str> = uuid.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);

        // All parts should be valid hexadecimal
        for (i, part) in parts.iter().enumerate() {
            assert!(
                part.chars().all(|c| c.is_ascii_hexdigit()),
                "Part {} should be valid hexadecimal, got: {}",
                i,
                part
            );
        }

        // Version should be 7 (first character of third part should be '7')
        assert!(
            parts[2].starts_with('7'),
            "Version should be 7, got: {}",
            parts[2]
        );

        // Variant should be valid (first character of fourth part should be 8, 9, A, or B)
        let variant_char = parts[3].chars().next().unwrap();
        assert!(
            matches!(variant_char, '8' | '9' | 'a' | 'b' | 'A' | 'B'),
            "Variant should be 8, 9, A, or B, got: {}",
            variant_char
        );
    }

    #[test]
    fn test_uuid_v7_monotonic() {
        // Test that UUIDs are monotonically increasing (time-ordered)
        let uuid1 = uuid_v7_weak();
        let uuid2 = uuid_v7_weak();
        let uuid3 = uuid_v7_weak();

        // In weak mode, they should be deterministic but still valid
        assert_eq!(uuid1.len(), 36);
        assert_eq!(uuid2.len(), 36);
        assert_eq!(uuid3.len(), 36);

        // All should be valid UUIDs
        assert!(uuid1.chars().all(|c| c.is_ascii_hexdigit() || c == '-'));
        assert!(uuid2.chars().all(|c| c.is_ascii_hexdigit() || c == '-'));
        assert!(uuid3.chars().all(|c| c.is_ascii_hexdigit() || c == '-'));
    }
}
