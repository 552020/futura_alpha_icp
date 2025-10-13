use crate::capsule::domain::{effective_perm_mask, Perm, PrincipalContext};
use crate::http::core_types::Acl;
use crate::memories::core::traits::Store;
use crate::memories::{CanisterEnv, StoreAdapter};
use crate::types::PersonRef;
use candid::Principal;

/// ACL adapter that wraps existing domain logic without importing domain code into HTTP layer
pub struct FuturaAclAdapter;

impl Acl for FuturaAclAdapter {
    fn can_view(&self, memory_id: &str, who: Principal) -> bool {
        // Create PrincipalContext for permission evaluation
        let ctx = PrincipalContext {
            principal: who,
            groups: vec![], // TODO: Get from user system if needed
            link: None,     // TODO: Extract from HTTP request if needed
            now_ns: ic_cdk::api::time(),
        };

        // Use the same pattern as existing memory operations
        let _env = CanisterEnv;
        let store = StoreAdapter;

        // Get all accessible capsules for the caller
        let accessible_capsules = store.get_accessible_capsules(&PersonRef::Principal(who));

        // Search for the memory across all accessible capsules
        for capsule_id in accessible_capsules {
            if let Some(memory) = store.get_memory(&capsule_id, &memory_id.to_string()) {
                // Use existing effective_perm_mask logic
                let perm_mask = effective_perm_mask(&memory, &ctx);
                return (perm_mask & Perm::VIEW.bits()) != 0;
            }
        }
        false
    }
}
