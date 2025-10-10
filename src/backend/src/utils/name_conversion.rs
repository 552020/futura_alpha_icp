//! Name conversion utilities for consistent title-to-name transformation
//! 
//! This module provides shared utilities for converting user-facing titles
//! to URL-safe identifiers across all entity types (Memory, Gallery, Folder, etc.)

/// Convert a user-facing title to a URL-safe identifier
/// 
/// # Examples
/// 
/// ```
/// use crate::utils::name_conversion::title_to_name;
/// 
/// assert_eq!(title_to_name("Vacation Photo 2024"), "vacation-photo-2024");
/// assert_eq!(title_to_name("My Dog's Birthday!"), "my-dogs-birthday");
/// assert_eq!(title_to_name("IMG_2024_12_19.jpg"), "img-2024-12-19-jpg");
/// assert_eq!(title_to_name("Beach Sunset 🌅"), "beach-sunset");
/// assert_eq!(title_to_name(""), "untitled");
/// ```
pub fn title_to_name(title: &str) -> String {
    if title.trim().is_empty() {
        return "untitled".to_string();
    }
    
    title
        .to_lowercase()
        .replace(" ", "-")           // spaces to hyphens
        .replace("_", "-")           // underscores to hyphens
        .replace(".", "-")           // dots to hyphens
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')  // only alphanumeric + hyphens
        .collect::<String>()
        .trim_matches('-')           // remove leading/trailing hyphens
        .to_string()
}

/// Generate a default name when no title is provided
pub fn generate_default_name(entity_type: &str, id: &str) -> String {
    format!("{}-{}", entity_type, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_to_name_conversion() {
        assert_eq!(title_to_name("Vacation Photo 2024"), "vacation-photo-2024");
        assert_eq!(title_to_name("My Dog's Birthday!"), "my-dogs-birthday");
        assert_eq!(title_to_name("IMG_2024_12_19.jpg"), "img-2024-12-19-jpg");
        assert_eq!(title_to_name("Beach Sunset 🌅"), "beach-sunset");
        assert_eq!(title_to_name("Test___Multiple___Underscores"), "test---multiple---underscores");
        assert_eq!(title_to_name("Test...Multiple...Dots"), "test---multiple---dots");
        assert_eq!(title_to_name("  Leading and Trailing Spaces  "), "leading-and-trailing-spaces");
        assert_eq!(title_to_name(""), "untitled");
        assert_eq!(title_to_name("   "), "untitled");
    }

    #[test]
    fn test_generate_default_name() {
        assert_eq!(generate_default_name("memory", "123"), "memory-123");
        assert_eq!(generate_default_name("gallery", "abc"), "gallery-abc");
        assert_eq!(generate_default_name("folder", "xyz"), "folder-xyz");
    }
}
