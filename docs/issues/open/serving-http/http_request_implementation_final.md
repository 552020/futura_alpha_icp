Here’s a crisp, **actionable implementation roadmap** for moving to **token-gated `http_request`** (primary) with **CustomImage (agent+blob)** as a fallback. It’s broken into phases, with owners, deliverables, acceptance criteria, and test plans.

---

# Roadmap: Private Image Delivery on ICP

## Guiding Principles

- Everything is **private** (thumbs included).
- No Web2 proxy for access control.
- **No per-image writes** just to read.
- Use **Next.js `<Image>`** where possible.
- Keep a **simple fallback** for edge cases.

---

## Phase 0 — Design Freeze (½ day)

**Decisions**

- Primary path: **Token-gated `http_request`** (stateless HMAC).
- Fallback: **CustomImage** (agent→bytes→`blob:`) for niche cases.
- Token TTL: **3 minutes** (configurable).
- Token scope: **per memory** (covers its thumbs/previews; originals too if needed).
- Streaming threshold: **2 MB** (use streaming callbacks ≥2 MB).

**Deliverables**

- Short design doc (2–3 pages) with token claims & headers.
- Update ADR / architecture notes.

**Acceptance**

- Tech lead sign-off on token shape + TTL + scope.

---

## Phase 1 — Canister Foundations (1–2 days)

**Tasks**

1. **Secret Management**

   - Generate a 32-byte secret on `init`/`post_upgrade`.
   - Store in stable memory; add simple rotation hook.

2. **Token Model (stateless)**

   - Payload (canonical JSON):

     ```json
     {
       "ver": 1,
       "scope": { "memory_id": "...", "variants": ["thumbnail", "preview", "original"] },
       "exp": 1738950000,
       "nonce": "96-bit-random"
     }
     ```

   - Signature: `HMAC_SHA256(secret, canonical_payload)`.

3. **Token Mint API (QUERY)**

   - `mint_http_token(scope, ttl_secs) -> token_b64`
   - Validates caller has VIEW on `memory_id`.
   - No writes (query only).

4. **Verification Helper**

   - `verify_http_token(token_b64, path, now) -> Ok(scope) | Err(reason)`

**Deliverables**

- `SecretStore` module.
- `TokenMint` (query) + `TokenVerify` helpers.
- Unit tests for sign/verify/expiry/scope.

**Acceptance**

- Unit tests green; fuzz test invalid tokens.

---

## Phase 2 — `http_request` Routes (2–3 days)

**Tasks**

1. **Router**

   - Paths: `/asset/{memory_id}/{variant}/{asset_id?}`
   - Variants: `thumbnail`, `preview`, `placeholder`, `original`
   - Parse `?token=...`

2. **Authorization**

   - Call `verify_http_token()`
   - Check path matches token scope (memory + variant [+ optional asset])

3. **Serving**

   - `<2 MB`: inline `HttpResponse` with bytes
   - `≥2 MB`: streaming strategy (callback)
     Re-verify token/session id on each chunk callback

4. **Headers**

   - `Cache-Control: private, no-store`
   - `Content-Type: {from metadata}`
   - Optional: `ETag: "<sha256-hex>"` (for client 304s within token TTL)

5. **Errors**

   - 401 (missing token), 403 (invalid/expired/scope mismatch), 404 (asset not found), 416 (bad range), 500 (unexpected)

**Deliverables**

- `http_request(HttpRequest) -> HttpResponse`
- `StreamingCallback` with re-verification
- Integration tests (`dfx` + HTTP)

**Acceptance**

- Downloads work for small/large files.
- Negative tests return correct codes.
- No writes during HTTP fetch.

---

## Phase 3 — Frontend Integration (1–2 days)

**Tasks**

1. **Token Fetch (QUERY)**

   - On page load (per memory view), call `mint_http_token({memory_id, variants}, 180s)`
   - Store `token` in state/context

2. **Next.js `<Image>` URLs**

   - Build URLs:
     `https://<canister>.icp0.io/asset/{memory_id}/{variant}?token=...&id={asset_id?}`
   - Add `remotePatterns` for `*.icp0.io` and `*.ic0.app` in `next.config.js`

3. **Fallback Component**

   - `CustomImage` (agent+blob) for very special cases
     (e.g., when `<Image>` isn’t suitable)

4. **Error UI**

   - Graceful placeholder on 401/403/404
   - Retry flow (re-mint token if expired)

**Deliverables**

- Token fetch hook: `useIcHttpToken(memoryId)`
- URL builder: `icAssetUrl({ memoryId, variant, assetId, token })`
- Updated components to use `<Image>` with tokenized URLs

**Acceptance**

- Thumbs/previews/originals render in lists and detail pages.
- Expired token → auto re-mint and retry works.

---

## Phase 4 — Performance & Hardening (2–3 days)

**Tasks**

1. **Streaming**

   - Verify chunk sizes, backpressure
   - Re-verify token/session per callback

2. **ETag/304 (optional)**

   - If desired within token lifetime (small wins)

3. **Logging**

   - Structured logs: `memory_id`, `variant`, size, duration, status, error

4. **Metrics**

   - `/metrics` or counters: total served, bytes served, avg latency, failures

5. **Rate Limiting (optional)**

   - Per token/session simple rate cap

6. **Security**

   - Strict path validation
   - Reject unknown variants
   - Token size/format limits

**Deliverables**

- Benchmark results (p50/p95 latency, throughput)
- Logs & metrics wired into dashboards

**Acceptance**

- p95 < 150ms for cached small assets; streaming stable for large.
- No memory pressure in canister; cycles within budget.

---

## Phase 5 — Rollout & Cleanup (1 day)

**Tasks**

- Feature flag and staged rollout (internal → beta → prod)
- Remove/deprecate old Web2 proxy routes
- Update docs and runbooks
- Add alerting for elevated 401/403/5xx

**Deliverables**

- Production deploy
- Deletion of legacy code
- Final docs: developer guide + troubleshooting

**Acceptance**

- No regressions in image grids
- Support confirms UX parity or better

---

## Testing Matrix

| Case                               | Expectation                               |
| ---------------------------------- | ----------------------------------------- |
| Valid token, small asset           | 200, correct bytes, `private, no-store`   |
| Expired token                      | 403                                       |
| Token for different memory/variant | 403                                       |
| Missing token                      | 401                                       |
| Nonexistent asset                  | 404                                       |
| Large asset (≥2 MB)                | Streams fully; consistent re-verification |
| Burst parallel loads               | No panics; stable latency                 |
| Malformed token                    | 401/403, not 500                          |

---

## Risk & Mitigation

- **Token leakage** → Short TTL, scoped claims, `no-store` headers.
- **Clock drift** → Accept small skew ±30s; use canister time for mint/verify.
- **Secret compromise** → Rotate on upgrade; short TTL limits damage.
- **Streaming callback abuse** → Re-verify session id + token scope per chunk.

---

## Owner Map

- **Canister work**: Mid-senior BE (Rust), code reviews by TL
- **Frontend work**: Senior FE (Next.js), review by TL
- **Perf/infra**: DevOps/Platform engineer
- **QA**: QA engineer + dev pairing for test matrix

---

## Success Criteria

- All images (thumbs, previews, originals) load via **`<Image>`** with private access.
- No Web2 proxy involved in access control.
- No per-image writes; only **query mint** + **read verify**.
- p95 latency improved vs Web2 proxy; memory stable; errors < 0.5%.

---

If you want, I can also add a short **task checklist** you can paste into your tracker (Jira/Linear) with ticket titles and acceptance criteria for each phase.
