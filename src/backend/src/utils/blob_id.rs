use regex::Regex;
use std::str::FromStr;

/// Parse a blob ID string into a u64, handling both "blob_<digits>" and "<digits>" formats
/// 
/// # Examples
/// ```
/// assert_eq!(parse_blob_id("blob_11410754707272541975").unwrap(), 11410754707272541975u64);
/// assert_eq!(parse_blob_id("11410754707272541975").unwrap(), 11410754707272541975u64);
/// assert!(parse_blob_id("blob_").is_err());
/// assert!(parse_blob_id("blob_abc").is_err());
/// assert!(parse_blob_id("  blob_123  ").is_ok());
/// ```
pub fn parse_blob_id(s: &str) -> Result<u64, String> {
    // Normalize - trim whitespace
    let raw = s.trim();
    
    // Accept "blob_<digits>" or "<digits>"
    // Precompile regex for better performance
    let re = Regex::new(r"^(?:blob_)?(\d+)$").unwrap();

    let caps = re.captures(raw).ok_or_else(|| format!("Invalid blob ID: '{}'", raw))?;
    let digits = caps.get(1).unwrap().as_str();
    u64::from_str(digits).map_err(|_| format!("Invalid blob ID: '{}'", raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blob_id_parser_accepts_both_formats() {
        assert_eq!(parse_blob_id("blob_11410754707272541975").unwrap(), 11410754707272541975u64);
        assert_eq!(parse_blob_id("11410754707272541975").unwrap(), 11410754707272541975u64);
        assert_eq!(parse_blob_id("blob_123").unwrap(), 123u64);
        assert_eq!(parse_blob_id("123").unwrap(), 123u64);
    }

    #[test]
    fn blob_id_parser_handles_whitespace() {
        assert_eq!(parse_blob_id("  blob_123  ").unwrap(), 123u64);
        assert_eq!(parse_blob_id("\t11410754707272541975\n").unwrap(), 11410754707272541975u64);
    }

    #[test]
    fn blob_id_parser_rejects_invalid_formats() {
        assert!(parse_blob_id("blob_").is_err());
        assert!(parse_blob_id("blob_abc").is_err());
        assert!(parse_blob_id("abc").is_err());
        assert!(parse_blob_id("blob_blob_123").is_err());
        assert!(parse_blob_id("").is_err());
        assert!(parse_blob_id("   ").is_err());
    }

    #[test]
    fn blob_id_parser_handles_edge_cases() {
        assert_eq!(parse_blob_id("blob_0").unwrap(), 0u64);
        assert_eq!(parse_blob_id("0").unwrap(), 0u64);
        // Test with max u64 value
        assert_eq!(parse_blob_id("18446744073709551615").unwrap(), u64::MAX);
        assert_eq!(parse_blob_id("blob_18446744073709551615").unwrap(), u64::MAX);
    }
}
