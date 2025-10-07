# SQLite Integration Phase 1: Technical Proposal

## 1. Integration Scope for Phase 1

### Target Data Structures

**Primary Tables:**

```sql
-- Memory metadata (replaces nested HashMap<String, Memory>)
CREATE TABLE memories (
    id TEXT PRIMARY KEY,
    capsule_id TEXT NOT NULL,
    title TEXT,
    description TEXT,
    memory_type TEXT NOT NULL,
    content_type TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    uploaded_at INTEGER NOT NULL,
    date_of_memory INTEGER,
    file_created_at INTEGER,
    parent_folder_id TEXT,
    tags TEXT, -- JSON array
    people_in_memory TEXT, -- JSON array
    location TEXT,
    memory_notes TEXT,
    created_by TEXT,
    is_public BOOLEAN NOT NULL,
    shared_count INTEGER NOT NULL DEFAULT 0,
    sharing_status TEXT NOT NULL,
    total_size INTEGER NOT NULL,
    asset_count INTEGER NOT NULL,
    thumbnail_url TEXT,
    primary_asset_url TEXT,
    has_thumbnails BOOLEAN NOT NULL,
    has_previews BOOLEAN NOT NULL,
    deleted_at INTEGER
);

-- Capsule-memory relationships (replaces nested HashMap)
CREATE TABLE capsule_memories (
    capsule_id TEXT NOT NULL,
    memory_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (capsule_id, memory_id),
    FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE
);

-- Memory tags (normalized for efficient searching)
CREATE TABLE memory_tags (
    memory_id TEXT NOT NULL,
    tag TEXT NOT NULL,
    PRIMARY KEY (memory_id, tag),
    FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE
);
```

**Indexes:**

```sql
-- Query optimization indexes
CREATE INDEX idx_memories_capsule_created ON memories(capsule_id, created_at DESC);
CREATE INDEX idx_memories_type ON memories(memory_type);
CREATE INDEX idx_memories_public ON memories(is_public);
CREATE INDEX idx_memory_tags_tag ON memory_tags(tag);
CREATE INDEX idx_memories_search ON memories(capsule_id, title, description);
```

### Excluded from Phase 1

- **Blob storage**: Keep in `StableBTreeMap<([u8; 32], u32), Vec<u8>>`
- **Capsule core data**: Keep in existing `StableBTreeMap<CapsuleId, Capsule>`
- **Session data**: Keep in current session storage
- **Admin data**: Keep in current admin storage

## 2. Data-Flow Adaptation

### Dual-Write Pattern

**Write Operations:**

```rust
impl MemoryStore {
    fn create_memory(&mut self, memory: Memory) -> Result<(), Error> {
        // 1. Write to existing StableBTreeMap (primary)
        self.stable_store.upsert(memory.id.clone(), memory.clone())?;

        // 2. Write to SQLite (secondary) - feature flag controlled
        if self.sqlite_enabled {
            if let Err(e) = self.sqlite_store.insert_memory_metadata(&memory) {
                // Log error but don't fail the operation
                ic_cdk::println!("SQLite write failed: {:?}", e);
            }
        }

        Ok(())
    }

    fn update_memory(&mut self, memory_id: &str, updates: MemoryUpdateData) -> Result<(), Error> {
        // 1. Update StableBTreeMap
        self.stable_store.update(&memory_id, |m| {
            // Apply updates to memory
        })?;

        // 2. Update SQLite if enabled
        if self.sqlite_enabled {
            self.sqlite_store.update_memory_metadata(memory_id, &updates)?;
        }

        Ok(())
    }
}
```

**Read Operations:**

```rust
impl MemoryStore {
    fn list_memories(&self, capsule_id: &str, filters: MemoryFilters) -> Page<MemoryHeader> {
        // Feature flag: SQLite vs manual iteration
        if self.sqlite_enabled && self.sqlite_healthy {
            self.sqlite_store.search_memories(capsule_id, filters)
        } else {
            // Fallback to current manual iteration
            self.stable_store.list_memories_manual(capsule_id, filters)
        }
    }
}
```

### Migration Script

**One-time Initialization:**

```rust
impl SqliteStore {
    fn initialize_from_stable_data(&mut self) -> Result<(), Error> {
        ic_cdk::println!("Initializing SQLite from stable data...");

        // 1. Create tables and indexes
        self.create_schema()?;

        // 2. Migrate existing memory metadata
        let capsules = self.stable_store.paginate(None, u32::MAX, Order::Asc);
        let mut migrated_count = 0;

        for capsule in capsules.items {
            for (memory_id, memory) in &capsule.memories {
                self.insert_memory_metadata(memory)?;
                migrated_count += 1;

                // Progress logging
                if migrated_count % 100 == 0 {
                    ic_cdk::println!("Migrated {} memories...", migrated_count);
                }
            }
        }

        ic_cdk::println!("Migration complete: {} memories migrated", migrated_count);
        Ok(())
    }
}
```

## 3. Operational Aspects

### Stable Memory Mapping

**SQLite File Storage:**

```rust
// Store SQLite database in stable memory
thread_local! {
    static SQLITE_DB: RefCell<Option<Connection>> = RefCell::new(None);
}

impl SqliteStore {
    fn init_from_stable_memory() -> Result<Self, Error> {
        // Map SQLite database to stable memory region
        let db_path = "stable://sqlite/memories.db";
        let connection = Connection::open(db_path)?;

        SQLITE_DB.with(|db| {
            *db.borrow_mut() = Some(connection);
        });

        Ok(SqliteStore {})
    }
}
```

**Upgrade Persistence:**

```rust
#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    // SQLite database automatically persists in stable memory
    // No explicit serialization needed
    ic_cdk::println!("SQLite database will persist automatically");
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    // Reinitialize SQLite connection
    if let Err(e) = SqliteStore::init_from_stable_memory() {
        ic_cdk::println!("SQLite initialization failed: {:?}", e);
        // Continue with SQLite disabled
    }
}
```

### Resource Impact Estimation

**Wasm Size Impact:**

- **`ic-rusqlite`**: ~500KB additional Wasm size
- **SQLite library**: ~1MB additional Wasm size
- **Total**: ~1.5MB increase (acceptable for canister limits)

**Cycle Impact:**

- **Query operations**: 10-50% reduction in cycles (indexed vs iteration)
- **Write operations**: 20-30% increase (dual-write overhead)
- **Memory usage**: ~10MB for SQLite buffer pool

### Consistency Monitoring

**Health Checks:**

```rust
impl SqliteStore {
    fn health_check(&self) -> HealthStatus {
        // 1. Test basic query
        match self.connection.execute("SELECT 1", []) {
            Ok(_) => HealthStatus::Healthy,
            Err(e) => {
                ic_cdk::println!("SQLite health check failed: {:?}", e);
                HealthStatus::Unhealthy
            }
        }
    }

    fn consistency_check(&self) -> Result<ConsistencyReport, Error> {
        // Compare record counts between StableBTreeMap and SQLite
        let stable_count = self.stable_store.count_memories();
        let sqlite_count = self.count_memories_sqlite()?;

        if stable_count != sqlite_count {
            return Ok(ConsistencyReport {
                status: ConsistencyStatus::Inconsistent,
                stable_count,
                sqlite_count,
                message: "Record count mismatch detected".to_string(),
            });
        }

        Ok(ConsistencyReport {
            status: ConsistencyStatus::Consistent,
            stable_count,
            sqlite_count,
            message: "Data consistency verified".to_string(),
        })
    }
}
```

## 4. Testing & Benchmarking Plan

### Performance Benchmarks

**Test Case: Memory Search + Pagination**

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;

    #[test]
    fn benchmark_memory_search() {
        let mut store = MemoryStore::new();

        // Setup: Create 1000 memories with various metadata
        for i in 0..1000 {
            let memory = create_test_memory(i);
            store.create_memory(memory);
        }

        // Benchmark: Search memories with filters and pagination
        let filters = MemoryFilters {
            capsule_id: "test_capsule".to_string(),
            memory_type: Some(MemoryType::Image),
            tags: Some(vec!["vacation".to_string()]),
            date_range: Some((start_date, end_date)),
        };

        // Current approach: Manual iteration
        let start = std::time::Instant::now();
        let manual_results = store.list_memories_manual("test_capsule", filters.clone());
        let manual_duration = start.elapsed();

        // SQLite approach: Indexed query
        let start = std::time::Instant::now();
        let sqlite_results = store.list_memories_sqlite("test_capsule", filters);
        let sqlite_duration = start.elapsed();

        // Assertions
        assert_eq!(manual_results.items.len(), sqlite_results.items.len());
        assert!(sqlite_duration < manual_duration);

        println!("Manual: {:?}, SQLite: {:?}", manual_duration, sqlite_duration);
    }
}
```

### Upgrade/Recovery Tests

**Recovery Test:**

```rust
#[test]
fn test_sqlite_recovery() {
    let mut store = MemoryStore::new();

    // 1. Create data in both stores
    let memory = create_test_memory(1);
    store.create_memory(memory);

    // 2. Simulate SQLite corruption
    store.corrupt_sqlite_database();

    // 3. Verify fallback to stable storage
    let results = store.list_memories("test_capsule", MemoryFilters::default());
    assert!(!results.items.is_empty());

    // 4. Test SQLite rebuild
    store.rebuild_sqlite_from_stable()?;
    let results = store.list_memories("test_capsule", MemoryFilters::default());
    assert!(!results.items.is_empty());
}
```

### Fallback Strategy

**Graceful Degradation:**

```rust
impl MemoryStore {
    fn list_memories(&self, capsule_id: &str, filters: MemoryFilters) -> Page<MemoryHeader> {
        // Try SQLite first
        if self.sqlite_enabled {
            match self.sqlite_store.search_memories(capsule_id, filters.clone()) {
                Ok(results) => return results,
                Err(e) => {
                    ic_cdk::println!("SQLite query failed, falling back to stable storage: {:?}", e);
                    // Mark SQLite as unhealthy for future requests
                    self.sqlite_healthy = false;
                }
            }
        }

        // Fallback to stable storage
        self.stable_store.list_memories_manual(capsule_id, filters)
    }
}
```

## Implementation Timeline

**Week 1-2: Core Infrastructure**

- Set up `ic-rusqlite` integration
- Create SQLite schema and basic CRUD operations
- Implement dual-write pattern

**Week 3-4: Migration & Testing**

- Build migration script from stable data
- Implement health checks and consistency monitoring
- Create performance benchmarks

**Week 5-6: Integration & Validation**

- Integrate with existing memory operations
- Test upgrade/rebuild scenarios
- Performance validation and optimization

## Success Metrics

- **Query Performance**: 50%+ improvement in memory search operations
- **Data Consistency**: 100% consistency between SQLite and stable storage
- **Reliability**: <1% fallback rate to stable storage
- **Resource Usage**: <2MB additional Wasm size, <20% cycle increase for writes

This Phase 1 approach provides a solid foundation for SQLite integration while maintaining backward compatibility and providing clear fallback mechanisms.
