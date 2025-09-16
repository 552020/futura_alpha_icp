# UploadThing Integration Questions for AI Research

## Context: Our Current System

We have a file upload system with the following characteristics:

### Current Architecture

- **Frontend**: Next.js with custom upload hooks (`useFileUpload`)
- **Backend**: Rust/ICP canister for some users, Neon database for others
- **Storage**: Multiple providers (Vercel Blob, ICP canister) with custom `StorageManager`
- **Upload Flow**: Hybrid approach - direct API uploads for small files, blob-first for large files
- **Authentication**: Complex onboarding vs authenticated user flows, ICP requires Internet Identity

### Key Requirements

- **ICP Integration**: Must support Internet Computer Protocol canister storage
- **Dual Storage**: Users can choose between Neon database or ICP canister
- **Custom Processing**: Image processing with multiple variants (original, display, thumb)
- **Onboarding Support**: Temporary users during onboarding process

## Questions for AI Research

### 1. UploadThing Open Source Scope

**Question**: "What exactly is open source in UploadThing? I know the frontend SDKs and API route handlers are open source, but what about the backend file processing? Can I find the source code for their file processing, storage management, and presigned URL generation on GitHub?"

**Follow-up**: "If the backend is closed source, what are the exact limitations? Can I self-host any part of UploadThing's infrastructure, or is it purely a SaaS service with open-source SDKs?"

### 2. Custom Storage Provider Integration

**Question**: "Can UploadThing be configured to use custom storage providers beyond AWS S3 and Cloudflare R2? Specifically, can I integrate it with Internet Computer Protocol (ICP) canister storage or other blockchain-based storage solutions?"

**Follow-up**: "What would be required to add support for a custom storage provider? Are there any examples or documentation for extending UploadThing's storage capabilities?"

### 3. Self-Hosting Possibilities

**Question**: "Is there any way to self-host UploadThing's file processing backend? Are there any open-source alternatives or forks that provide the backend infrastructure? What would be needed to run UploadThing completely on my own infrastructure?"

**Follow-up**: "If self-hosting isn't possible, are there any open-source file upload services that provide similar functionality but with full backend control?"

### 4. Hybrid Integration Strategy

**Question**: "Given that UploadThing's backend is closed source, what's the best way to integrate it alongside an existing custom upload system? Can I use UploadThing for some users (e.g., standard blob storage) while keeping my custom ICP integration for others?"

**Follow-up**: "What would be the technical challenges of running both systems in parallel? Are there any examples of projects that have successfully integrated UploadThing with custom storage backends?"

### 5. File Processing Customization

**Question**: "Can I customize UploadThing's file processing pipeline? For example, can I add custom image processing steps, modify the file optimization algorithms, or add support for custom file types and transformations?"

**Follow-up**: "If customization isn't possible, what are the alternatives for implementing custom file processing while still using UploadThing's upload infrastructure?"

### 6. Authentication and User Management

**Question**: "How does UploadThing handle complex authentication flows? Can I integrate it with custom authentication systems like Internet Identity for ICP users, or does it only work with standard web authentication?"

**Follow-up**: "What are the limitations for supporting temporary users during onboarding processes? Can UploadThing handle users who aren't fully authenticated yet?"

### 7. Cost and Pricing Model

**Question**: "What is UploadThing's pricing model? Are there any costs for using their open-source SDKs, or do you only pay when using their managed backend service?"

**Follow-up**: "What are the typical costs for a file upload service with moderate usage? Are there any free tiers or open-source alternatives that might be more cost-effective?"

### 8. Migration and Compatibility

**Question**: "If I wanted to migrate from a custom upload system to UploadThing, what would be the migration path? Can I gradually adopt UploadThing while keeping my existing system running?"

**Follow-up**: "What are the compatibility considerations? Would I need to change my database schema, API endpoints, or frontend components?"

## Specific Technical Questions

### 9. API Integration Details

**Question**: "Can you find the exact API endpoints and request/response formats that UploadThing uses? What are the technical requirements for integrating with their backend service?"

### 10. Open Source Alternatives

**Question**: "Are there any fully open-source alternatives to UploadThing that provide similar functionality? I'm looking for solutions that include both frontend components and backend file processing that I can self-host and customize."

## Expected Outcomes

Based on the research, we want to determine:

1. **Feasibility**: Can we integrate UploadThing with our ICP requirements?
2. **Architecture**: What would the hybrid system look like?
3. **Limitations**: What can't we do with UploadThing?
4. **Alternatives**: Are there better open-source solutions?
5. **Migration Path**: How would we implement this integration?

## Current Understanding (To Be Verified)

- UploadThing frontend SDKs are open source
- UploadThing backend is closed source and managed
- Integration requires `UPLOADTHING_TOKEN`
- No database changes needed for basic integration
- Presigned URL pattern for direct-to-storage uploads

## Research Results

Based on AI research with internet access, here are the precise, sourced answers:

---

### 1. UploadThing Open Source Scope

**Answer**:

- **Open source**: The SDKs you install (client hooks/components and server "file routes"). MIT-licensed on GitHub (`pingdotgg/uploadthing`). ([GitHub](https://github.com/pingdotgg/uploadthing))
- **Closed source**: The managed backend (storage, dashboard, metering, auth for uploads, webhooks infra). Docs describe using their tokened backend and webhooks, not a self-hosted core. ([docs.uploadthing.com](https://docs.uploadthing.com/api-reference/server))
- You will not find their storage layer / presign service / dashboard code in GitHub—only the SDKs and docs site.

**Follow-up**: Self-hosting their backend isn't officially supported; there are old community threads about reverse-engineering, but that's not a supported path. ([GitHub Issue #78](https://github.com/pingdotgg/uploadthing/issues/78))

---

### 2. Custom Storage Provider Integration

**Answer**:

- Officially, files go to UploadThing's storage in regions you choose (default AWS us-west-2). There's no "bring your own storage" switch in docs. ([docs.uploadthing.com](https://docs.uploadthing.com/concepts/regions-acl))
- No documented adapter API to plug in ICP canister or other blockchains directly. The SDK assumes UT's backend issues presigned URLs and receives webhooks.

**What it would take**: Treat UT and your ICP flow as separate lanes (see #4). For true custom storage inside UT, there's no official extension point published.

---

### 3. Self-Hosting Possibilities

**Answer**:

- **Official answer**: None. The docs provide server SDK + webhooks that depend on `UPLOADTHING_TOKEN` (i.e., their SaaS). ([docs.uploadthing.com](https://docs.uploadthing.com/api-reference/server))
- Community hacks exist (unsupported). Proceed at your own risk.

**Open-source, self-hostable alternatives (full control)**:

- tus protocol + `tusd` (server) + Uppy (client) for resumable uploads
- Supabase Storage (OSS) or MinIO (S3-compatible) with your own presign endpoints

---

### 4. Hybrid Integration Strategy

**Answer**:

- **Yes**: Run UT for "standard blob" users and keep your ICP path for ICP users. UT's file routes are just API handlers; you decide who can use which route via middleware. ([docs.uploadthing.com](https://docs.uploadthing.com/file-routes))
- **Challenges**:
  - Two sources of truth for files; standardize a file table with a provider enum and normalized metadata (UT file URL/id vs ICP asset id)
  - Different auth models (NextAuth/cookies vs Internet Identity) inside your file-route middleware
  - Different processing pipelines

No public case study of UT + custom blockchain storage; but Payload CMS shows community adapters for picking UT vs other storage, proving the "parallel providers" pattern. ([Payload](https://payloadcms.com/docs/upload/storage-adapters))

---

### 5. File Processing Customization

**Answer**:

- **Client-side**: You can pre-process/rename before requesting presigned URLs (`onBeforeUploadBegin`) and track progress. ([docs.uploadthing.com](https://docs.uploadthing.com/api-reference/react))
- **Server-side**: UT calls your route's `onUploadComplete` via webhook; you can trigger your own processing (e.g., image variants, video jobs) in your infra. UT doesn't expose its internal transform pipeline to customize.

If you need full control over transforms (e.g., custom codecs, ICP writes), run your own processor after `onUploadComplete` (workers/queues) or skip UT and use tus/S3-multipart + your own pipeline.

---

### 6. Authentication & User Management

**Answer**:

- You add middleware to file routes; you can integrate whatever auth you want (sessions, tokens, Internet Identity via your own verification) before allowing an upload. ([docs.uploadthing.com](https://docs.uploadthing.com/uploading-files))
- Temporary/onboarding users: Supported as long as your middleware logic accepts them; UT doesn't impose user models—your app decides.

---

### 7. Cost & Pricing

**Answer**:

- The SDKs are free (MIT). You pay only when you use the managed backend. ([GitHub](https://github.com/pingdotgg/uploadthing))
- Current pricing highlights: Free 2 GB app; $10/mo 100 GB app; usage-based tiers. ([uploadthing.com](https://uploadthing.com))

---

### 8. Migration & Compatibility

**Answer**:

- Gradual adoption works: Add UT file routes alongside your existing endpoints; wire only some user segments/flows to UT. ([docs.uploadthing.com](https://docs.uploadthing.com/file-routes))
- DB/schema: Keep your current tables; store UT file metadata (id/url/size/mime) at `onUploadComplete` and unify with your ICP rows via a "storage_provider" column. ([docs.uploadthing.com](https://docs.uploadthing.com/api-reference/ut-api))
- Frontend: You can keep your `useFileUpload` and call UT's `uploadFiles()` under the hood for the UT path, or mount their component where convenient.

---

### 9. API Integration Details

**Answer**:

- There's no public, stable, raw HTTP REST spec to bypass the SDK. You integrate via:
  - Server "file routes" you define with `createUploadthing` (the SDK generates the handler)
  - A server token `UPLOADTHING_TOKEN`
  - A webhook/`onUploadComplete` callback to your absolute route URL
  - Optional UT API helper (`UTApi`) for file ops from your server

---

### 10. Fully Open-Source Alternatives

**Answer**:

- tus + `tusd` + Uppy (battle-tested, resumable, S3/R2/MinIO friendly)
- Supabase Storage (self-hostable) + client SDK; or MinIO + your own presign endpoints
- Directus (self-host) with file handling; or a DIY Next.js + S3 multipart flow

---

## What This Means for Our Goals

### 1. Feasibility (ICP)

UT can't store into ICP natively. Use a hybrid: UT for "blob users", your ICP pipeline for "ICP users." Gate via file-route middleware and a `storage_provider` flag.

### 2. Architecture (Hybrid Sketch)

- **Frontend**: Your `useFileUpload` chooses path A (UT) or B (ICP) by user/account policy
- **Backend**:
  - Path A (UT): UT file route + `onUploadComplete` → write file row (provider=UT)
  - Path B (ICP): Your existing upload endpoint → canister write (provider=ICP)
- **Processing**: Shared worker consumes events from both paths to create `original|display|thumb` variants

### 3. Limitations

- No official BYO storage / custom provider inside UT
- No server-side transform hooks you can configure inside UT's pipeline; do your own after the webhook

### 4. Alternatives

If you need total control + resumability for 10 GB files with ICP, tus/tusd + Uppy is likely the cleanest OSS route.

### 5. Migration Path

- Start by routing only Neon-backed, non-ICP users to UT; keep ICP path untouched
- Add a `storage_provider` column and a thin adapter in your upload service
- Gradually expand UT usage if the DX/costs make sense

## Recommendation

Based on this research, a **hybrid approach** is the most viable path:

1. Keep your existing ICP integration for ICP users
2. Use UploadThing for standard blob storage users
3. Implement a unified file management layer that handles both providers
4. Consider tus/tusd + Uppy as a fully open-source alternative if you need complete control
