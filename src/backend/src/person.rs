use crate::types::PersonRef;
use ic_cdk::api::msg_caller;

impl PersonRef {
    /// Create a PersonRef from the current caller
    pub fn from_caller() -> Self {
        PersonRef::Principal(msg_caller())
    }

    /// Create an opaque PersonRef (for deceased/non-principal subjects)
    #[allow(dead_code)] // Used in tests
    pub fn opaque(id: String) -> Self {
        PersonRef::Opaque(id)
    }

    /// Check if this PersonRef matches the current caller
    #[allow(dead_code)] // Used in tests
    pub fn is_caller(&self) -> bool {
        match self {
            PersonRef::Principal(p) => *p == msg_caller(),
            PersonRef::Opaque(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_ref_from_caller() {
        // Note: This test can't actually call msg_caller() in unit tests
        // but we can test the opaque creation
        let opaque_ref = PersonRef::opaque("test_id".to_string());
        assert!(matches!(opaque_ref, PersonRef::Opaque(_)));
    }

    #[test]
    fn test_person_ref_opaque() {
        let id = "deceased_person_123".to_string();
        let person_ref = PersonRef::opaque(id.clone());

        match person_ref {
            PersonRef::Opaque(ref opaque_id) => assert_eq!(opaque_id, &id),
            PersonRef::Principal(_) => panic!("Expected Opaque variant"),
        }
    }

    #[test]
    fn test_person_ref_is_caller() {
        // Test with opaque reference (should always return false)
        let opaque_ref = PersonRef::opaque("test_id".to_string());
        assert!(!opaque_ref.is_caller());

        // Note: Testing is_caller() with Principal references is not possible in unit tests
        // because msg_caller() is not available outside of canister context.
        // This functionality should be tested in integration tests.
    }
}
