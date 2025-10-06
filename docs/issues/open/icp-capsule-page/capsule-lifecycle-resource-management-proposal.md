# 🚀 Capsule Lifecycle & Resource Management Enhancement Proposal

## **Current Capsule Type (Before Enhancement)**

```rust
pub struct Capsule {
    pub id: String,                                          // unique capsule identifier
    pub subject: PersonRef,                                  // who this capsule is about
    pub owners: HashMap<PersonRef, OwnerState>,              // 1..n owners (usually 1)
    pub controllers: HashMap<PersonRef, ControllerState>,    // delegated admins (full control)
    pub connections: HashMap<PersonRef, Connection>,         // social graph
    pub connection_groups: HashMap<String, ConnectionGroup>, // organized connection groups
    pub memories: HashMap<String, Memory>,                   // content
    pub galleries: HashMap<String, Gallery>,                 // galleries (collections of memories)
    pub created_at: u64,
    pub updated_at: u64,
    pub bound_to_neon: bool,    // Neon database binding status
    pub inline_bytes_used: u64, // Track inline storage consumption
}
```

**Current Limitations:**

- ❌ **No expiration management** - capsules exist indefinitely
- ❌ **No storage quotas** - unlimited storage per capsule
- ❌ **No resource tracking** - no visibility into usage
- ❌ **No tiered access** - all capsules treated equally
- ❌ **No lifecycle management** - no cleanup or archival

## **Enhanced Capsule Type (After Enhancement)**

```rust
pub struct Capsule {
    // ... existing fields ...
    pub id: String,
    pub subject: PersonRef,
    pub owners: HashMap<PersonRef, OwnerState>,
    pub controllers: HashMap<PersonRef, ControllerState>,
    pub connections: HashMap<PersonRef, Connection>,
    pub connection_groups: HashMap<String, ConnectionGroup>,
    pub memories: HashMap<String, Memory>,
    pub galleries: HashMap<String, Gallery>,
    pub created_at: u64,
    pub updated_at: u64,
    pub bound_to_neon: bool,
    pub inline_bytes_used: u64,

    // NEW: Lifecycle and Resource Management
    pub expiration_date: Option<u64>,        // When this capsule expires (None = never expires)
    pub auto_renewal: bool,                  // Auto-renew before expiration
    pub grace_period_days: u32,             // Grace period after expiration
    pub allocated_storage_bytes: u64,       // Total storage quota allocated to this capsule
    pub used_storage_bytes: u64,            // Current storage usage
    pub allocated_cycles: u64,              // Cycles allocated for this capsule's operations
    pub consumed_cycles: u64,               // Cycles consumed by this capsule's operations
    pub storage_tier: StorageTier,          // Storage tier (Free, Basic, Premium, Enterprise)
    pub cycle_billing_enabled: bool,        // Whether to track cycles
    pub cycle_consumption_rate: f64,        // Cycles per operation
}
```

**New Capabilities:**

- ✅ **Expiration management** - capsules can have expiration dates
- ✅ **Storage quotas** - configurable storage limits per capsule
- ✅ **Resource tracking** - visibility into storage and cycle usage
- ✅ **Tiered access** - different storage tiers (Free, Basic, Premium, Enterprise)
- ✅ **Lifecycle management** - automatic cleanup and archival
- ✅ **Auto-renewal** - automatic extension before expiration
- ✅ **Grace periods** - data recovery windows after expiration
- ✅ **Cycle billing** - per-operation cost tracking

## **Summary**

Proposal to enhance capsule types with lifecycle management (expiration dates) and resource allocation (storage quotas, cycle tracking) for better user experience and business model support.

## **Problem Statement**

Currently, capsules are **stateless containers** without:

- **Expiration management** - no way to set capsule lifetimes
- **Storage quotas** - no limits on capsule size
- **Resource tracking** - no visibility into storage/cycle usage
- **Tiered access** - no differentiation between free/premium users

## **Proposed Solution**

### **1. Lifecycle Management**

```rust
pub struct Capsule {
    // ... existing fields ...

    // NEW: Lifecycle Management
    pub expiration_date: Option<u64>,        // When capsule expires (None = never)
    pub auto_renewal: bool,                  // Auto-renew before expiration
    pub grace_period_days: u32,             // Grace period after expiration
}
```

**Use Cases:**

- **Temporary capsules** for events/conferences
- **Subscription-based access** with renewal
- **Data retention policies** for compliance

### **2. Storage Quota System**

```rust
pub struct Capsule {
    // ... existing fields ...

    // NEW: Storage Management
    pub allocated_storage_bytes: u64,       // Total quota (e.g., 1GB, 10GB, 100GB)
    pub used_storage_bytes: u64,            // Current usage
    pub storage_tier: StorageTier,          // Free, Basic, Premium, Enterprise
    pub storage_warnings: Vec<StorageWarning>, // Usage alerts
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum StorageTier {
    Free { max_bytes: u64 },           // 100MB free tier
    Basic { max_bytes: u64 },          // 1GB basic tier
    Premium { max_bytes: u64 },        // 10GB premium tier
    Enterprise { max_bytes: u64 },     // Unlimited enterprise
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct StorageWarning {
    pub warning_type: StorageWarningType,
    pub threshold_percentage: u8,      // 80%, 90%, 95%
    pub triggered_at: u64,
    pub message: String,
}
```

**Use Cases:**

- **Freemium model** with storage limits
- **Upgrade prompts** when approaching limits
- **Enterprise unlimited** storage

### **3. Cycle Tracking & Billing**

```rust
pub struct Capsule {
    // ... existing fields ...

    // NEW: Cycle Management
    pub allocated_cycles: u64,              // Cycles allocated to this capsule
    pub consumed_cycles: u64,              // Cycles consumed by operations
    pub cycle_billing_enabled: bool,        // Whether to track cycles
    pub cycle_consumption_rate: f64,        // Cycles per operation
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct CycleUsage {
    pub capsule_id: String,
    pub operation_type: OperationType,
    pub cycles_consumed: u64,
    pub timestamp: u64,
    pub cost_per_cycle: f64,               // In ICP tokens
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum OperationType {
    MemoryCreate,
    MemoryRead,
    MemoryUpdate,
    MemoryDelete,
    CapsuleCreate,
    CapsuleUpdate,
    // ... other operations
}
```

**Use Cases:**

- **Pay-per-use billing** for operations
- **Resource consumption tracking** per capsule
- **Cost optimization** insights

## **Technical Questions for ICP Expert**

### **Q1: Cycles in ICP Context**

- **Are cycles canister-level or can we track per-capsule consumption?**
- **How do we measure cycle consumption for individual operations?**
- **Is there a standard way to track cycle usage in ICP applications?**

### **Q2: Storage Quotas Implementation**

- **How do we efficiently track storage usage across memories/assets?**
- **Should we implement soft limits (warnings) vs hard limits (blocking)?**
- **How do we handle storage cleanup when quotas are exceeded?**

### **Q3: Expiration Management**

- **Should expired capsules be automatically deleted or just marked as expired?**
- **How do we handle grace periods and data recovery?**
- **Should we implement automatic backup before expiration?**

## **✅ ICP Expert Review - COMPLETED**

### **Q1: Cycles in ICP Context - ✅ ANSWERED**

- **Are cycles canister-level or can we track per-capsule consumption?**
  - **ANSWER**: Cycles are **canister-level only**. Each canister has a single cycles account. Per-capsule tracking must be implemented in application logic.
- **How do we measure cycle consumption for individual operations?**
  - **ANSWER**: Use **performance counter API** to count instructions executed by specific functions. Must instrument code to measure per-operation usage.
- **Is there a standard way to track cycle usage in ICP applications?**
  - **ANSWER**: **No built-in per-entity tracking**. Must implement in application logic using instruction counters.

### **Q2: Storage Quotas Implementation - ✅ ANSWERED**

- **How do we efficiently track storage usage across memories/assets?**
  - **ANSWER**: Protocol charges for **total canister memory** (heap + stable). Must implement own logic to sum sizes per capsule.
- **Should we implement soft limits (warnings) vs hard limits (blocking)?**
  - **ANSWER**: **Application-level decision**. Protocol doesn't enforce per-capsule quotas.
- **How do we handle storage cleanup when quotas are exceeded?**
  - **ANSWER**: **No automatic cleanup**. Must implement blocking, deletion, or cleanup routines in business logic.

### **Q3: Expiration Management - ✅ ANSWERED**

- **Should expired capsules be automatically deleted or just marked as expired?**
  - **ANSWER**: **Application choice**. Protocol doesn't enforce object-level lifecycles.
- **How do we handle grace periods and data recovery?**
  - **ANSWER**: **Must implement in application logic**. No built-in grace periods.
- **Should we implement automatic backup before expiration?**
  - **ANSWER**: **No built-in backup**. Must implement at application level, possibly exporting to another canister.

### **🎯 Key ICP Expert Takeaways:**

- ✅ **Cycles and storage are canister-level** - per-capsule tracking is application-level
- ✅ **Instruction/cycle measurement is possible** using performance counters
- ✅ **All quotas, warnings, cleanup are application-level** - not protocol-enforced
- ✅ **Lifecycle management is entirely application-level** - no protocol hooks

## **✅ Tech Lead Review - COMPLETED**

### **Q1: Database Schema Impact - ✅ ANSWERED**

- **How do these changes affect the Neon database schema?**
  - **ANSWER**: Need new tables: `capsule_tiers`, `capsule_usage_daily`, `storage_warning_events`, `billing_events`
  - **ANSWER**: Add columns to `capsules`: `expiration_date`, `grace_days`, `tier`, `allocated_storage_bytes`, `used_storage_bytes`, `auto_renewal`, `cycle_billing_enabled`
- **Do we need new tables for cycle tracking and storage warnings?**
  - **ANSWER**: Yes - `capsule_usage_daily` for usage tracking, `storage_warning_events` for warnings
- **How do we handle migration of existing capsules?**
  - **ANSWER**: Add new fields as `Option` in Candid, set defaults in `post_upgrade`, use `StateVersion` enum for backfill

### **Q2: API Design - ✅ ANSWERED**

- **Should we add new endpoints for quota management?**
  - **ANSWER**: Yes - `GET /capsules/:id/usage`, `POST /capsules/:id/upgrade`, `POST /capsules/:id/renew`
  - **ANSWER**: Admin endpoints: `PUT /capsules/:id/quota`, `POST /capsules/:id/compaction`
- **How do we handle quota exceeded errors gracefully?**
  - **ANSWER**: Standardize errors: `QUOTA_EXCEEDED`, `CAPSULE_EXPIRED`, `PAYMENT_REQUIRED`
- **Do we need admin endpoints for quota management?**
  - **ANSWER**: Yes - quota management, compaction, and monitoring endpoints

### **Q3: Business Logic - ✅ ANSWERED**

- **How do we implement tier upgrades/downgrades?**
  - **ANSWER**: Immediate quota change; on downgrade, enforce new limit on next write; keep grace if over limit
- **What happens when storage quota is exceeded?**
  - **ANSWER**: Block writes; allow reads & deletes; surface upgrade CTA
- **How do we handle cycle billing and payment integration?**
  - **ANSWER**: Start with monthly subscription (tiered storage); add pay-per-op using instruction/byte estimates later

### **🎯 Key Tech Lead Takeaways:**

- ✅ **Architecture choice**: Start with single canister + estimates, move to one-canister-per-capsule for Premium/Enterprise
- ✅ **Implementation approach**: Use timers for lifecycle, performance counters for compute tracking
- ✅ **Database design**: Specific tables and columns defined
- ✅ **API design**: Complete endpoint specification with error handling
- ✅ **Migration strategy**: Backward-compatible with state versioning

## **🚀 Tech Lead Implementation Guidance**

### **Architecture Decision: Two Models**

1. **Single Canister + Estimates** (Start here)
   - ✅ **Pros**: Simplest ops, fewer canisters
   - ❌ **Cons**: Cycles only at canister level → per-capsule billing is app-level estimates
2. **One Canister Per Capsule** (Premium/Enterprise)
   - ✅ **Pros**: True isolation; real cycles/storage per capsule via `canister_status`
   - ❌ **Cons**: More canisters to manage; upgrades/top-ups at scale

### **Concrete Implementation Steps**

#### **1. Lifecycle Management (Expiry + Grace)**

```rust
use ic_cdk_timers::{set_timer_interval, TimerId};
use std::time::Duration;

#[ic_cdk::init]
fn init() { register_timers(); }

#[ic_cdk::post_upgrade]
fn post_upgrade() { register_timers(); }

fn register_timers() {
    let _id: TimerId = set_timer_interval(Duration::from_secs(3600), || housekeeping_tick());
}

fn housekeeping_tick() {
    // Process expirations in small batches to stay under instruction limits
}
```

#### **2. Storage Quotas**

```rust
fn ensure_quota(c: &Capsule, add_bytes: u64) -> Result<(), CapsuleError> {
    let next = c.used_storage_bytes.saturating_add(add_bytes);
    if next > c.allocated_storage_bytes {
        return Err(CapsuleError::QuotaExceeded { used: next, allocated: c.allocated_storage_bytes });
    }
    Ok(())
}
```

#### **3. Compute/Cycles Tracking**

```rust
use ic_cdk::api::performance_counter;

pub fn meter_op<F, R>(capsule_id: CapsuleId, op: OperationType, f: F) -> R
where F: FnOnce() -> R {
    let start = performance_counter();
    let r = f();
    let instr = performance_counter() - start;
    persist_usage(capsule_id, op, instr);
    r
}
```

### **Database Schema (Neon)**

```sql
-- New tables
CREATE TABLE capsule_tiers (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    max_bytes BIGINT NOT NULL,
    features JSONB
);

CREATE TABLE capsule_usage_daily (
    capsule_id VARCHAR(255) NOT NULL,
    day DATE NOT NULL,
    bytes_used BIGINT NOT NULL,
    instr_exec BIGINT NOT NULL,
    est_cycles_compute BIGINT NOT NULL,
    est_cycles_storage BIGINT NOT NULL,
    PRIMARY KEY (capsule_id, day)
);

CREATE TABLE storage_warning_events (
    id SERIAL PRIMARY KEY,
    capsule_id VARCHAR(255) NOT NULL,
    pct INTEGER NOT NULL,
    triggered_at TIMESTAMP NOT NULL,
    message TEXT NOT NULL
);

CREATE TABLE billing_events (
    id SERIAL PRIMARY KEY,
    capsule_id VARCHAR(255) NOT NULL,
    kind VARCHAR(50) NOT NULL,
    amount_cycles BIGINT NOT NULL,
    amount_fiat DECIMAL(10,2),
    at TIMESTAMP NOT NULL
);

-- Add columns to existing capsules table
ALTER TABLE capsules ADD COLUMN expiration_date TIMESTAMP;
ALTER TABLE capsules ADD COLUMN grace_days INTEGER DEFAULT 7;
ALTER TABLE capsules ADD COLUMN tier VARCHAR(50) DEFAULT 'Free';
ALTER TABLE capsules ADD COLUMN allocated_storage_bytes BIGINT DEFAULT 104857600; -- 100MB
ALTER TABLE capsules ADD COLUMN used_storage_bytes BIGINT DEFAULT 0;
ALTER TABLE capsules ADD COLUMN auto_renewal BOOLEAN DEFAULT false;
ALTER TABLE capsules ADD COLUMN cycle_billing_enabled BOOLEAN DEFAULT false;
```

### **API Endpoints**

```rust
// User endpoints
GET /capsules/:id/usage          // Current usage + projections
POST /capsules/:id/upgrade       // Tier change
POST /capsules/:id/renew         // Renewal

// Admin endpoints
PUT /capsules/:id/quota          // Set quota
POST /capsules/:id/compaction    // Cleanup
```

### **Error Handling**

```rust
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum CapsuleError {
    QuotaExceeded { used: u64, allocated: u64 },
    CapsuleExpired { expired_at: u64 },
    PaymentRequired { amount: u64 },
    // ... existing errors
}
```

## **Implementation Plan**

### **Phase 1: Core Types (1-2 days)**

- [ ] Add new fields to `Capsule` struct
- [ ] Create `StorageTier` and `CycleUsage` types
- [ ] Update `CapsuleInfo` to include new fields
- [ ] Add validation for storage quotas

### **Phase 2: Storage Tracking (2-3 days)**

- [ ] Implement storage usage calculation
- [ ] Add storage warning system
- [ ] Create quota enforcement logic
- [ ] Add storage cleanup for expired capsules

### **Phase 3: Cycle Tracking (2-3 days)**

- [ ] Implement cycle consumption measurement
- [ ] Add cycle usage logging
- [ ] Create cycle billing calculations
- [ ] Add cycle quota enforcement

### **Phase 4: API Integration (1-2 days)**

- [ ] Add new endpoints for quota management
- [ ] Update existing endpoints with quota checks
- [ ] Add admin endpoints
- [ ] Add error handling for quota exceeded

## **Migration Strategy**

### **Existing Capsules**

```rust
// Default values for existing capsules
impl Default for Capsule {
    fn default() -> Self {
        Self {
            // ... existing fields ...
            expiration_date: None,                    // Never expires
            allocated_storage_bytes: 100 * 1024 * 1024, // 100MB free tier
            used_storage_bytes: 0,                     // Start with 0 usage
            allocated_cycles: 1_000_000,              // 1M cycles
            consumed_cycles: 0,                       // Start with 0 consumption
            storage_tier: StorageTier::Free { max_bytes: 100 * 1024 * 1024 },
        }
    }
}
```

### **Backward Compatibility**

- All existing capsules get default values
- No breaking changes to existing APIs
- Gradual rollout with feature flags

## **Business Impact**

### **Revenue Opportunities**

- **Freemium model** with storage limits
- **Subscription tiers** with different quotas
- **Pay-per-use** cycle billing
- **Enterprise unlimited** plans

### **User Experience**

- **Clear storage usage** visibility
- **Upgrade prompts** when approaching limits
- **Automatic renewal** options
- **Cost transparency** for operations

## **Risks & Mitigation**

### **Risk 1: Performance Impact**

- **Mitigation**: Lazy calculation of storage usage, caching
- **Monitoring**: Track API response times

### **Risk 2: Data Loss**

- **Mitigation**: Grace periods, backup before expiration
- **Monitoring**: Alert on quota exceeded

### **Risk 3: Complex Billing**

- **Mitigation**: Start simple, iterate based on feedback
- **Monitoring**: Track cycle consumption patterns

## **Success Metrics**

- **Storage quota adoption** rate
- **Cycle consumption** patterns
- **User upgrade** conversion rate
- **System performance** impact

## **Next Steps**

1. **Tech Lead Review** - Architecture and implementation approach
2. **ICP Expert Review** - Cycle tracking and ICP-specific considerations
3. **Business Review** - Revenue model and user experience
4. **Implementation** - Start with Phase 1 core types

---

## **📋 Expert Review Summary**

### **✅ ICP Expert Review - COMPLETED**

- **Feasibility**: ✅ **CONFIRMED** - All features are technically feasible
- **Implementation**: ✅ **Application-level** - No protocol-level support needed
- **Key Insight**: All per-capsule resource management must be implemented in business logic

### **✅ Tech Lead Review - COMPLETED**

- **Architecture**: ✅ **DECIDED** - Start with single canister + estimates, move to one-canister-per-capsule for Premium/Enterprise
- **Implementation**: ✅ **DETAILED** - Complete code examples, database schema, API design
- **Migration**: ✅ **PLANNED** - Backward-compatible with state versioning

### **🎯 Ready for Implementation**

1. **✅ All expert reviews complete** - Both ICP and Tech Lead guidance received
2. **✅ Implementation plan detailed** - Code examples, database schema, API endpoints
3. **✅ Architecture decided** - Single canister start, one-canister-per-capsule for Premium
4. **✅ Migration strategy** - Backward-compatible with existing capsules

### **🚀 Implementation Priority**

1. **Phase 1**: Core types and basic tracking (1-2 days)
2. **Phase 2**: Storage quotas and warnings (2-3 days)
3. **Phase 3**: Cycle tracking and billing (2-3 days)
4. **Phase 4**: API integration and admin endpoints (1-2 days)

---

**Status**: ✅ **READY FOR IMPLEMENTATION**  
**Priority**: 🔥 **High** - Core business model feature  
**Estimated Effort**: 1-2 weeks  
**Dependencies**: ✅ **All expert reviews complete**
