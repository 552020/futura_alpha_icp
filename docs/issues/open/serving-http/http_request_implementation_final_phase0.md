Perfect. Let’s lock this in right here — you, me, and our imaginary senior/devs.
Here’s the decision set; we’ll mark ✅ what we choose:

---

## Phase 0 — Design Freeze (Team Decision)

| Item                          | Decision                              | Notes                                                                        |
| ----------------------------- | ------------------------------------- | ---------------------------------------------------------------------------- |
| **Primary Path**              | ✅ Token-gated `http_request`         | Stateless HMAC token verified by backend                                     |
| **Fallback**                  | ✅ CustomImage (agent → blob)         | For rare cases where `<Image>` not used                                      |
| **Token Scope**               | ✅ Per Memory                         | Token allows access to all variants of one memory (thumb, preview, original) |
| **TTL (Lifetime)**            | ✅ 3 minutes                          | Short-lived; enough for page load + lazy loading                             |
| **Streaming Threshold**       | ✅ 2 MB                               | Inline below; streaming callbacks ≥ 2 MB                                     |
| **Headers**                   | ✅ `Cache-Control: private, no-store` | Prevent browser or CDN persistence                                           |
| **Secret Rotation**           | ✅ On upgrade                         | New HMAC secret generated at post_upgrade                                    |
| **ETag/304 Support**          | ❌ Not now                            | Maybe later after baseline works                                             |
| **Token Signing Algo**        | ✅ HMAC-SHA256                        | Simple, fast, deterministic                                                  |
| **Public vs Private**         | ✅ All Private                        | No certified assets; all require token                                       |
| **Streaming Re-Verification** | ✅ Yes                                | Each callback re-verifies token                                              |
| **Next.js Integration**       | ✅ Direct ICP URL with `?token=…`     | `<Image>` keeps full optimization                                            |

---

### Summary

We’ll implement:

- **Stateless HMAC tokens**
- **Short TTL (3 min)**
- **Per-memory scope**
- **Private HTTP responses**
- **Streaming for > 2 MB**

No Web2 proxy, no writes, no certification.

---

✅ **Decision Freeze: Done**
Proceed to **Phase 1 — Canister Foundations** (secret management + token mint/verify).
