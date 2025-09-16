# UploadThing Alternatives: Open-Source File Upload Solutions

## Overview

This document explores fully open-source alternatives to UploadThing that provide complete control over both frontend and backend file upload infrastructure. These solutions are particularly relevant for projects that need custom storage providers (like ICP canisters) or complete self-hosting capabilities.

## 1. tus Protocol + tusd + Uppy

### What is tus?

tus is an open protocol for resumable file uploads. It's designed to enable reliable, resumable uploads even in the face of network issues.

### Architecture

- **tusd**: Go-based server implementation
- **Uppy**: JavaScript client library with React components
- **tus protocol**: HTTP-based protocol for resumable uploads

### Key Features

- ✅ **Resumable uploads** - Resume interrupted uploads
- ✅ **Chunked uploads** - Upload large files in chunks
- ✅ **Progress tracking** - Real-time upload progress
- ✅ **Multiple storage backends** - S3, GCS, Azure, local filesystem
- ✅ **Self-hostable** - Complete control over infrastructure
- ✅ **Custom storage adapters** - Can be extended for ICP integration

### Integration with Your System

```typescript
// Frontend with Uppy
import Uppy from "@uppy/core";
import Dashboard from "@uppy/dashboard";
import Tus from "@uppy/tus";

const uppy = new Uppy().use(Dashboard, { target: "#app", inline: true }).use(Tus, {
  endpoint: "/api/upload",
  // Custom headers for ICP authentication
  headers: {
    Authorization: `Bearer ${icpToken}`,
  },
});

// Backend with tusd
// Can be configured to store to custom backends
```

### Pros

- Battle-tested and widely adopted
- Excellent for large file uploads
- Highly customizable
- Strong community support
- Can be extended for ICP integration

### Cons

- More complex setup than UploadThing
- Requires more infrastructure management
- Less "batteries included" than managed solutions

### ICP Integration Potential

- **High**: Can create custom storage adapters for ICP canisters
- **Implementation**: Extend tusd with custom storage backend
- **Authentication**: Can integrate Internet Identity tokens

---

## 2. Supabase Storage (Self-Hostable)

### What is Supabase Storage?

Supabase provides an open-source alternative to Firebase, including a self-hostable storage solution.

### Architecture

- **Supabase Storage**: S3-compatible storage with additional features
- **Supabase Client**: JavaScript/TypeScript client libraries
- **PostgreSQL**: Metadata storage
- **GoTrue**: Authentication system

### Key Features

- ✅ **Self-hostable** - Run on your own infrastructure
- ✅ **S3-compatible** - Works with existing S3 tools
- ✅ **Built-in authentication** - User management included
- ✅ **Real-time subscriptions** - WebSocket support
- ✅ **Image transformations** - Built-in image processing
- ✅ **Row Level Security** - Fine-grained access control

### Integration with Your System

```typescript
// Frontend
import { createClient } from "@supabase/supabase-js";

const supabase = createClient(url, key);

// Upload file
const { data, error } = await supabase.storage.from("memories").upload(`${userId}/${file.name}`, file);

// Custom ICP integration
const uploadToICP = async (file: File) => {
  // Your existing ICP upload logic
  const icpResult = await icpUploadService.uploadFile(file);

  // Store metadata in Supabase
  await supabase.from("files").insert({
    name: file.name,
    icp_canister_id: icpResult.canisterId,
    storage_provider: "icp",
  });
};
```

### Pros

- Complete self-hosting capability
- Built-in authentication and user management
- Real-time features
- Good documentation and community
- Can be extended for custom storage

### Cons

- PostgreSQL dependency
- More complex than simple file upload solutions
- Requires more infrastructure setup

### ICP Integration Potential

- **Medium**: Can store ICP metadata and URLs
- **Implementation**: Hybrid approach - ICP for storage, Supabase for metadata
- **Authentication**: Can integrate with existing auth systems

---

## 3. MinIO + Custom Presign Endpoints

### What is MinIO?

MinIO is a high-performance, S3-compatible object storage server.

### Architecture

- **MinIO Server**: S3-compatible storage server
- **Custom API**: Your own presigned URL generation
- **Frontend**: Custom upload components or libraries

### Key Features

- ✅ **S3-compatible** - Works with existing S3 tools and libraries
- ✅ **High performance** - Optimized for speed
- ✅ **Self-hostable** - Complete control
- ✅ **Multi-tenant** - Support for multiple applications
- ✅ **Encryption** - Built-in encryption support
- ✅ **Customizable** - Can be extended for specific needs

### Integration with Your System

```typescript
// Backend - Custom presign endpoint
app.post("/api/upload/presign", async (req, res) => {
  const { fileName, fileType, userId } = req.body;

  // Generate presigned URL for MinIO
  const presignedUrl = await minioClient.presignedPutObject(
    "memories",
    `${userId}/${fileName}`,
    24 * 60 * 60 // 24 hours
  );

  res.json({ presignedUrl, key: `${userId}/${fileName}` });
});

// Frontend - Direct upload to MinIO
const uploadFile = async (file: File) => {
  // Get presigned URL
  const { presignedUrl, key } = await fetch("/api/upload/presign", {
    method: "POST",
    body: JSON.stringify({
      fileName: file.name,
      fileType: file.type,
      userId: currentUser.id,
    }),
  });

  // Upload directly to MinIO
  await fetch(presignedUrl, {
    method: "PUT",
    body: file,
  });

  // Store metadata in your database
  await fetch("/api/memories", {
    method: "POST",
    body: JSON.stringify({
      name: file.name,
      storageKey: key,
      storageProvider: "minio",
    }),
  });
};
```

### Pros

- S3-compatible (familiar API)
- High performance
- Self-hostable
- Can be extended for custom needs
- Good for large-scale deployments

### Cons

- Requires more setup and configuration
- Need to build custom upload components
- More infrastructure management

### ICP Integration Potential

- **Medium**: Can be used alongside ICP storage
- **Implementation**: Hybrid approach - MinIO for standard files, ICP for specific use cases
- **Authentication**: Custom authentication integration

---

## 4. Directus (Self-Host)

### What is Directus?

Directus is an open-source headless CMS with built-in file handling capabilities.

### Architecture

- **Directus Core**: Node.js-based API server
- **Directus Studio**: Admin interface
- **File Storage**: Configurable storage adapters
- **Database**: PostgreSQL, MySQL, SQLite, etc.

### Key Features

- ✅ **Headless CMS** - API-first approach
- ✅ **File management** - Built-in file handling
- ✅ **Self-hostable** - Complete control
- ✅ **Multiple storage adapters** - S3, GCS, local, etc.
- ✅ **User management** - Built-in authentication
- ✅ **Real-time** - WebSocket support
- ✅ **Extensible** - Custom extensions and hooks

### Integration with Your System

```typescript
// Directus configuration
const directus = new Directus("https://your-directus-instance.com");

// Upload file
const formData = new FormData();
formData.append("file", file);

const response = await directus.files.create(formData);

// Custom ICP integration via hooks
// In Directus hooks, you can trigger ICP uploads
directus.hooks.before("files.create", async (input) => {
  if (input.storage === "icp") {
    // Upload to ICP canister
    const icpResult = await icpUploadService.uploadFile(input.file);
    input.icp_canister_id = icpResult.canisterId;
  }
});
```

### Pros

- Complete CMS solution
- Built-in file management
- Self-hostable
- Extensible with hooks
- Good for content-heavy applications

### Cons

- More than just file upload (full CMS)
- May be overkill for simple upload needs
- Requires learning Directus concepts

### ICP Integration Potential

- **High**: Can be extended with custom hooks and storage adapters
- **Implementation**: Custom storage adapter for ICP
- **Authentication**: Can integrate with existing auth systems

---

## 5. DIY Next.js + S3 Multipart

### Custom Implementation

Build your own upload solution using Next.js API routes and S3 multipart uploads.

### Architecture

- **Next.js API Routes**: Custom upload endpoints
- **S3 Multipart**: For large file uploads
- **Custom Frontend**: Your own upload components
- **Database**: Your existing database for metadata

### Key Features

- ✅ **Complete control** - Every aspect is customizable
- ✅ **S3 multipart** - Efficient large file uploads
- ✅ **Custom logic** - Exactly what you need
- ✅ **ICP integration** - Can implement custom ICP logic
- ✅ **No dependencies** - Only what you choose

### Integration with Your System

```typescript
// API Route - Multipart upload initiation
app.post("/api/upload/initiate", async (req, res) => {
  const { fileName, fileType, fileSize } = req.body;

  // Initiate multipart upload
  const multipartUpload = await s3.createMultipartUpload({
    Bucket: "your-bucket",
    Key: `uploads/${userId}/${fileName}`,
    ContentType: fileType,
  });

  res.json({
    uploadId: multipartUpload.UploadId,
    key: `uploads/${userId}/${fileName}`,
  });
});

// API Route - Upload part
app.post("/api/upload/part", async (req, res) => {
  const { uploadId, partNumber, key } = req.body;

  // Upload part to S3
  const uploadPart = await s3.uploadPart({
    Bucket: "your-bucket",
    Key: key,
    PartNumber: partNumber,
    UploadId: uploadId,
    Body: req.body.file,
  });

  res.json({
    ETag: uploadPart.ETag,
    PartNumber: partNumber,
  });
});

// Frontend - Multipart upload
const uploadLargeFile = async (file: File) => {
  const chunkSize = 5 * 1024 * 1024; // 5MB chunks
  const chunks = Math.ceil(file.size / chunkSize);

  // Initiate upload
  const { uploadId, key } = await fetch("/api/upload/initiate", {
    method: "POST",
    body: JSON.stringify({
      fileName: file.name,
      fileType: file.type,
      fileSize: file.size,
    }),
  });

  // Upload parts
  const parts = [];
  for (let i = 0; i < chunks; i++) {
    const start = i * chunkSize;
    const end = Math.min(start + chunkSize, file.size);
    const chunk = file.slice(start, end);

    const { ETag } = await fetch("/api/upload/part", {
      method: "POST",
      body: JSON.stringify({
        uploadId,
        partNumber: i + 1,
        key,
        file: chunk,
      }),
    });

    parts.push({ ETag, PartNumber: i + 1 });
  }

  // Complete upload
  await fetch("/api/upload/complete", {
    method: "POST",
    body: JSON.stringify({ uploadId, key, parts }),
  });
};
```

### Pros

- Complete control over every aspect
- Can implement exactly what you need
- No external dependencies
- Can integrate ICP seamlessly
- Optimized for your specific use case

### Cons

- More development time required
- Need to handle edge cases yourself
- More testing and maintenance
- Need to implement features from scratch

### ICP Integration Potential

- **Very High**: Can implement custom ICP logic directly
- **Implementation**: Custom API routes for ICP uploads
- **Authentication**: Full control over authentication flow

---

## Comparison Matrix

| Solution           | Self-Hostable | ICP Integration | Complexity | Performance | Community |
| ------------------ | ------------- | --------------- | ---------- | ----------- | --------- |
| tus + tusd + Uppy  | ✅            | High            | Medium     | High        | Strong    |
| Supabase Storage   | ✅            | Medium          | Medium     | High        | Strong    |
| MinIO + Custom API | ✅            | Medium          | High       | Very High   | Good      |
| Directus           | ✅            | High            | Medium     | Good        | Strong    |
| DIY Next.js + S3   | ✅            | Very High       | High       | High        | N/A       |

## Recommendations for Your Use Case

### For ICP Integration Priority:

1. **DIY Next.js + S3** - Complete control, can implement ICP directly
2. **tus + tusd + Uppy** - Extensible, can create custom storage adapters
3. **Directus** - Extensible with hooks, can add ICP storage adapter

### For Ease of Implementation:

1. **Supabase Storage** - Good balance of features and simplicity
2. **tus + tusd + Uppy** - Well-documented, good community support
3. **MinIO + Custom API** - S3-compatible, familiar patterns

### For Performance:

1. **MinIO + Custom API** - Optimized for high performance
2. **DIY Next.js + S3** - Can optimize for your specific needs
3. **tus + tusd + Uppy** - Good performance with resumable uploads

## Conclusion

For your specific requirements (ICP integration, custom authentication, self-hosting), the **tus + tusd + Uppy** solution offers the best balance of:

- ✅ Complete self-hosting capability
- ✅ High extensibility for ICP integration
- ✅ Strong community support and documentation
- ✅ Battle-tested reliability
- ✅ Good performance for large files

The **DIY Next.js + S3** approach would give you the most control but requires significantly more development time.

**Supabase Storage** could work well if you're willing to use a hybrid approach (ICP for specific users, Supabase for others).

## Next Steps

1. **Evaluate tus + tusd + Uppy** - Test the extensibility for ICP integration
2. **Consider hybrid approach** - Use different solutions for different user types
3. **Prototype implementation** - Build a small proof-of-concept with your preferred solution
4. **Performance testing** - Compare upload speeds and reliability with your current system
