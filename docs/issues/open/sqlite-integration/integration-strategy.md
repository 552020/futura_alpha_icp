# **SQLite Integration Strategy (ic-rusqlite)**

## **Purpose**

This memo defines our strategy for introducing **SQLite via `ic-rusqlite`** into the ICP canister architecture.
The goal is to augment the current **stable-structure database** with a **relational query layer**, improving query flexibility, filtering, and relationship management—without disrupting existing storage or data integrity.

---

## **1. Integration Model**

### **Parallel / Derived Query System**

SQLite will be introduced as a **parallel system**, not as a replacement at first.

- **Canonical source of truth:** existing stable-memory structures (`StableBTreeMap`, `StableVec`, `StableCell`).
- **SQLite layer:** acts as a **derived index** for complex reads, filters, and relationships.

#### **Write Path**

1. Apply mutation to the stable-memory store (canonical).
2. Perform a best-effort SQLite write in the same update call.
3. On SQLite error, log and mark entry for reindexing (no user-facing failure).

#### **Read Path**

- **Phase 1 (Shadow Mode):** continue serving reads from stable storage; execute equivalent SQLite queries in background for verification.
- **Phase 2 (Selective Read):** enable SQLite reads per-endpoint via feature flag.
- **Phase 3 (Full Read):** cut over entirely to SQLite for selected datasets.

#### **Feature Flags**

- `sqlite_enabled` — global switch.
- `sqlite_write_through` — enable/disable dual writes.
- `sqlite_shadow_read_pct` — percentage of reads to cross-check in shadow mode.

#### **Recovery**

- Periodic job (heartbeat or upgrade hook) can rebuild SQLite state from stable memory if inconsistencies or version mismatches are detected.

---

## **2. Schema Alignment with Postgres / `schema.ts`**

Our web2 backend already defines a **Postgres schema** via `schema.ts` (Drizzle).
To maintain parity and simplify synchronization, the SQLite schema on ICP will be aligned with it wherever possible.

### **Portable Schema Subset**

We will define a **cross-database contract** representing the shared model between Postgres and SQLite.

**Common subset (safe types):**

| Postgres Type     | SQLite Type                             | Notes                        |
| ----------------- | --------------------------------------- | ---------------------------- |
| `uuid`            | `TEXT`                                  | store canonical string       |
| `timestamptz`     | `INTEGER` (Unix ms) or `TEXT` (ISO8601) | choose consistent convention |
| `jsonb`           | `TEXT`                                  | store as serialized JSON     |
| `enum`            | `TEXT` + optional `CHECK`               | enforce app-side             |
| `bigint`          | `INTEGER` or `TEXT`                     | depending on range           |
| `boolean`         | `INTEGER` (0/1)                         |                              |
| `varchar`, `text` | `TEXT`                                  |                              |

**Indexes:**
Replicate only the indexes used in hot queries (search, filtering, pagination).
Name them consistently across both systems.

### **Schema Versioning**

- Maintain a `schema_version` table within the SQLite database.
- Store `schema_hash` (computed from schema definition) for validation.
- Embedded SQLite migrations will be shipped in the canister binary.
- The canister validates and upgrades schema automatically at startup.

---

## **3. Implementation Abstraction**

Define a unified repository interface to toggle between KV and SQLite modes.

```rust
pub trait MemoriesRepo {
    fn insert(&mut self, memory: Memory) -> Result<()>;
    fn search(&self, query: Query) -> Result<Vec<MemoryHeader>>;
}

pub struct KvRepo { /* existing stable structures */ }

pub struct KvPlusSqliteRepo {
    kv: KvRepo,
    db: SqliteConn,
    mode: Mode, // Shadow, Partial, Full
}
```

**Dual-write example:**

```rust
fn insert(&mut self, memory: Memory) -> Result<()> {
    self.kv.insert(&memory)?;
    if flags::sqlite_write_through() {
        if let Err(e) = sqlite_insert(&self.db, &memory) {
            log::warn!("SQLite insert failed: {e}");
            mark_for_reindex(memory.id);
        }
    }
    Ok(())
}
```

**Read example:**

```rust
fn search(&self, q: Query) -> Result<Vec<MemoryHeader>> {
    if flags::read_from_sqlite() {
        sqlite_search(&self.db, &q)
    } else {
        self.kv.search(&q)
    }
}
```

---

## **4. Integration with Web2 Schema**

To keep both environments consistent:

1. Use the same entity naming and field types as in `schema.ts`.
2. Maintain a small JSON or YAML manifest that defines:

   - tables, columns, and types;
   - index definitions;
   - current schema hash.

3. Optionally auto-generate a `schema.sqlite.sql` file at build time from the manifest.

This keeps the Postgres and SQLite models traceable to the same schema contract, reducing drift and easing synchronization logic.

---

## **5. Evaluation & Rollout Plan**

| Phase       | Goal               | Description                                                                             |
| ----------- | ------------------ | --------------------------------------------------------------------------------------- |
| **Phase 0** | Prototype          | Create SQLite schema for `memories` and run shadow queries vs KV results.               |
| **Phase 1** | Metadata migration | Dual-write metadata (e.g., `MemoryHeader`) into SQLite. Measure Wasm size & cycle cost. |
| **Phase 2** | Selective adoption | Enable SQLite reads for search and pagination endpoints.                                |
| **Phase 3** | Expansion          | Migrate relationship queries (`capsule` ↔ `memory`).                                    |
| **Phase 4** | Monitoring         | Track performance, cycles, schema version, and rebuild times.                           |

---

## **6. Summary**

- **SQLite** will serve as a _derived relational layer_ within the canister.
- The canonical data remains in **stable-memory KV**.
- Integration will proceed **incrementally** via dual writes and shadow reads.
- Schema design will follow the **portable subset** of our Postgres schema.
- Long term, this will simplify query logic, improve performance, and enable more expressive data relationships inside the ICP environment.

---

Would you like me to add a short “Next Steps” section (task list for the upcoming sprint) at the end so you can drop it straight into Linear or Notion?
