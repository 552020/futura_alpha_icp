# **ICP `http_request` Implementation – Developer Input Sheet**

**Purpose:**
We're implementing direct HTTP asset serving in our backend canister (so browsers can load images over `https://<canister>.icp0.io/...`).
This document gathers all required technical details about our current storage model and runtime behavior before finalizing the implementation.

---

## **1. General Canister Information**

| Question                                           | Answer                                                                                                            |
| -------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| **Canister name / module**                         | `backend` (main backend canister)                                                                                 |
| **Rust crate entry file**                          | `src/backend/src/lib.rs`                                                                                          |
| **Existing exposed methods**                       | `memories_read`, `memories_read_asset`, `asset_get_by_id`, `memories_list_assets`, `blob_read`, `blob_read_chunk` |
| **Is this canister public or private by default?** | Private by default – requires Internet Identity authentication and capsule ownership                              |

---

## **2. Asset Storage Model**

| Question                                                    | Answer                                                                                                            |
| ----------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| **Where are image bytes stored?**                           | Three storage types: 1) Inline (Vec<u8> in memory), 2) ICP blob storage (chunked), 3) External storage (S3, etc.) |
| **Key type**                                                | String-based locators: `"blob_{id}"` for ICP storage, asset_id for inline                                         |
| **Value type**                                              | `MemoryAssetInline` (Vec<u8>), `MemoryAssetBlobInternal` (BlobRef), `MemoryAssetBlobExternal` (external ref)      |
| **Do you store content type / MIME type along with bytes?** | Yes, in `AssetMetadataBase.mime_type` field                                                                       |
| **Are images chunked?**                                     | Yes, for blob storage. Chunks stored in `STABLE_BLOB_STORE` with `(pmid_hash, page_idx)` keys                     |
| **Typical size of assets?**                                 | Inline: <2MB, Blob: up to 10MB+, External: unlimited                                                              |
| **Asset variants per image**                                | **4 variants per image**: Original, Thumbnail, Preview, Placeholder (for optimization)                            |
| **Primary HTTP serving assets**                             | **Thumbnails, Previews, Placeholders** (optimized for display). Originals available for download only             |

---

## **3. Access Control**

| Question                                                                             | Answer                                                                                                                                                                                                                    |
| ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Are assets public or private by default?**                                         | Private by default. **Public assets**: Thumbnails, Previews, Placeholders (optimized for display). **Private assets**: Original images (download only)                                                                    |
| **How do you determine if a user can access an asset?**                              | Via `effective_perm_mask()` using capsule ownership and access entries. User must own capsule and have VIEW permission                                                                                                    |
| **Do you use Internet Identity or custom session tokens in browser?**                | Internet Identity for authentication, no custom tokens                                                                                                                                                                    |
| **Should the `http_request` endpoint be publicly accessible or require validation?** | Mixed: `/asset/{id}/thumbnail` public, `/asset/{id}/preview` public, `/asset/{id}/placeholder` public, `/asset/{id}/{index}` private with user validation, `/asset/{id}/original/{asset_id}` private with user validation |

---

## **4. Current Backend API Pattern**

| Question                                                      | Answer                                                                                                                                                    |
| ------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **How do you currently fetch image bytes from the canister?** | `memories_read_asset(memory_id, asset_index)`, `asset_get_by_id(memory_id, asset_id)`, `memories_list_assets(memory_id)` → returns `MemoryAssetData` enum |
| **Is the image served inline or streamed chunk-by-chunk?**    | Inline for small assets, chunked for large blob assets                                                                                                    |
| **Do you already support partial reads (range requests)?**    | No, but `blob_read_chunk()` exists for chunked reading                                                                                                    |
| **Is the image hash (ETag equivalent) stored or derivable?**  | Yes, stored in `AssetMetadataBase.sha256` field                                                                                                           |

---

## **5. Expected HTTP Behavior**

| Question                                                        | Answer                                                                                                                                                            |
| --------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Which paths should be handled by the canister?**              | `/asset/{memory_id}/{asset_index}`, `/asset/{memory_id}/{asset_type}` (thumbnail, preview, placeholder), `/asset/{memory_id}/original/{asset_id}` (download only) |
| **Expected MIME types**                                         | `image/jpeg`, `image/png`, `image/webp`, `image/gif`                                                                                                              |
| **Do you need to support query params like `?w=600&q=80`?**     | **No** - optimized variants (thumbnail, preview, placeholder) are pre-generated. Next.js Image component will use appropriate variant automatically               |
| **Should the canister respond with streaming for large files?** | Yes, for assets >2MB using existing chunked reading                                                                                                               |
| **Should the canister set caching headers?**                    | Yes: public assets (1 year), private assets (no-cache)                                                                                                            |
| **Should private assets skip certification?**                   | Yes, private assets use skip certification with user validation                                                                                                   |

---

## **6. Certification and Libraries**

| Question                                                                        | Answer                                                         |
| ------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| **Do we plan to use `ic-certified-assets` or `ic-asset-certification`?**        | `ic-http-certification` for custom implementation              |
| **Will this canister also handle non-asset API calls (business logic)?**        | Yes, this is the main backend canister with all business logic |
| **Do we have an existing certified assets canister or plan to merge?**          | No, implementing directly in main backend canister             |
| **Do we use the `http_certification` crate already (anywhere in the project)?** | No, will be added as new dependency                            |

---

## **7. Frontend Integration**

| Question                                                                                | Answer                                                                                                |
| --------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| **How will the frontend reference these assets?**                                       | Direct ICP URLs in Next.js Image component using optimized variants (thumbnail, preview, placeholder) |
| **Should URLs look like** `https://<canister>.icp0.io/assets/<id>` **or custom paths?** | `/asset/{memory_id}/thumbnail`, `/asset/{memory_id}/preview`, `/asset/{memory_id}/placeholder` format |
| **Do we need signed URLs or temporary tokens?**                                         | No, user principal validation via Internet Identity                                                   |
| **Will these assets be used in `<Image>` (Next.js) or `<img>` tags?**                   | Next.js `<Image>` component with optimization                                                         |
| **Do we need CORS headers for requests from web2 frontend?**                            | No, direct ICP serving doesn't need CORS                                                              |

---

## **8. Future-Proofing & Migration**

| Question                                                                                | Answer                                                                                                                                   |
| --------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| **Will we migrate all assets to certified format eventually?**                          | Yes, **public optimized variants** (thumbnails, previews, placeholders) will be certified, **private originals** will skip certification |
| **Any plan to shard assets across multiple canisters?**                                 | No, single backend canister for now                                                                                                      |
| **Do we need backward compatibility with existing blob routes (`/api/blobs/...`)?**     | No, old backend proxy routes are being removed                                                                                           |
| **Do we expect third-party clients (outside Futura frontend) to consume these assets?** | No, assets are private to users                                                                                                          |

---

## **9. Optional Features (Mark if Yes)**

| Feature                                           | Needed? | Notes                                                              |
| ------------------------------------------------- | ------- | ------------------------------------------------------------------ |
| Support JSON/text responses (for debug endpoints) | ☑       | Health check endpoint                                              |
| Image transformations (resize, format)            | ☐       | Pre-generated optimized variants (thumbnail, preview, placeholder) |
| Range / streaming responses                       | ☑       | For large blob assets                                              |
| ETag / caching validation                         | ☑       | Using SHA256 from metadata                                         |
| HTTPS-only / redirect                             | ☐       | ICP boundary nodes handle HTTPS                                    |
| Logging per request                               | ☑       | For debugging and monitoring                                       |
| Metrics endpoint (`/metrics`)                     | ☑       | Basic canister metrics                                             |

---

## **10. Example Asset (Optional)**

Please provide one example of an existing stored image record.

```json
{
  "memory_id": "c6f07efb-4e4f-73c0-c6f0-0000000073c0",
  "assets": [
    {
      "asset_id": "original_12345",
      "asset_type": "Original",
      "content_type": "image/jpeg",
      "size_bytes": 2048576,
      "storage": "MemoryAssetBlobInternal",
      "is_public": false,
      "metadata": {
        "name": "vacation_photo.jpg",
        "mime_type": "image/jpeg",
        "width": 1920,
        "height": 1080,
        "sha256": [123, 45, 67, ...]
      }
    },
    {
      "asset_id": "thumbnail_12345",
      "asset_type": "Thumbnail",
      "content_type": "image/jpeg",
      "size_bytes": 128734,
      "storage": "MemoryAssetInline",
      "is_public": true,
      "metadata": {
        "name": "vacation_photo_thumb.jpg",
        "mime_type": "image/jpeg",
        "width": 300,
        "height": 200,
        "sha256": [234, 56, 78, ...]
      }
    },
    {
      "asset_id": "preview_12345",
      "asset_type": "Preview",
      "content_type": "image/webp",
      "size_bytes": 256789,
      "storage": "MemoryAssetBlobInternal",
      "is_public": true,
      "metadata": {
        "name": "vacation_photo_preview.webp",
        "mime_type": "image/webp",
        "width": 800,
        "height": 600,
        "sha256": [345, 67, 89, ...]
      }
    },
    {
      "asset_id": "placeholder_12345",
      "asset_type": "Placeholder",
      "content_type": "image/jpeg",
      "size_bytes": 2048,
      "storage": "MemoryAssetInline",
      "is_public": true,
      "metadata": {
        "name": "vacation_photo_placeholder.jpg",
        "mime_type": "image/jpeg",
        "width": 20,
        "height": 15,
        "sha256": [456, 78, 90, ...]
      }
    }
  ]
}
```

---

## **11. Implementation Requirements Summary**

### **Core Requirements:**

- ✅ **URL Format**: `/asset/{memory_id}/thumbnail`, `/asset/{memory_id}/preview`, `/asset/{memory_id}/placeholder`, `/asset/{memory_id}/original/{asset_id}`
- ✅ **Authentication**: Internet Identity + capsule ownership validation
- ✅ **Storage Integration**: Use existing `memories_read_asset()`, `asset_get_by_id()`, `memories_list_assets()`, and `blob_read()` methods
- ✅ **Access Control**: Integrate with `effective_perm_mask()` system
- ✅ **Certification**: **Public optimized variants** (thumbnails, previews, placeholders) certified, **private originals** skip certification
- ✅ **Streaming**: Support chunked reading for large blob assets
- ✅ **Caching**: Public optimized variants (1 year), private originals (no-cache)
- ✅ **MIME Types**: Support common image formats (JPEG, PNG, WebP, GIF)
- ✅ **Asset Variants**: Serve **4 variants per image**: Original (download), Thumbnail (display), Preview (display), Placeholder (loading)

### **Performance Targets:**

- **Response Time**: 10-50ms (vs 250-900ms backend proxy)
- **Throughput**: Handle concurrent requests efficiently
- **Memory Usage**: Minimal memory overhead for asset serving

### **Security Requirements:**

- **Privacy**: User data never leaves ICP network
- **Access Control**: Strict permission validation per asset
- **Authentication**: Internet Identity principal validation
- **Data Integrity**: SHA256 verification for asset integrity
- **Asset Separation**: **Public optimized variants** (thumbnails, previews, placeholders) vs **private originals** (download only)

---

### Once this is filled out:

I will:

1. Generate a **complete production-ready Rust implementation** of `http_request`,
2. Include both certified (public) and non-certified (private) routes,
3. Handle streaming, MIME detection, and optional caching.

---

**Status**: ✅ **COMPLETE** - All technical requirements gathered and documented.

**Next Step**: Implement the `http_request` method in `src/backend/src/lib.rs` based on these specifications.
