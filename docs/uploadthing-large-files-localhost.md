# UploadThing Large File Uploads & Localhost Development

## The Problem You're Describing

You're absolutely right about this challenge! Here's the issue:

### **Large File Upload Flow:**

1. **Client** requests presigned URL from UploadThing
2. **Client** uploads directly to UploadThing's storage (bypassing your server)
3. **UploadThing** calls your `onUploadComplete` webhook
4. **Your server** processes the completion

### **The Localhost Problem:**

- UploadThing needs to call your `onUploadComplete` webhook
- In development, your server runs on `localhost:3000`
- UploadThing's servers can't reach `localhost` from the internet
- This breaks the completion callback flow

## How UploadThing Handles This

### **1. Presigned URL Pattern**

```typescript
// UploadThing generates presigned URLs for direct uploads
const presignedUrl = await generateSignedURL(`${ingestUrl}/${key}`, apiKey, {
  ttlInSeconds: routeOptions.presignedURLTTL,
  data: {
    "x-ut-identifier": appId,
    "x-ut-route": routeSlug,
    "x-ut-callback": `${yourServerUrl}/api/uploadthing`, // This is the problem!
  },
});
```

### **2. The Callback Issue**

The `x-ut-callback` header contains your server URL, which UploadThing uses to call `onUploadComplete`. In development:

- **Production**: `https://yourdomain.com/api/uploadthing` ✅
- **Development**: `http://localhost:3000/api/uploadthing` ❌ (not reachable)

## Solutions for Local Development

### **Option 1: Use ngrok (Recommended)**

```bash
# Install ngrok
npm install -g ngrok

# Expose your local server
ngrok http 3000

# Use the ngrok URL in your environment
NEXT_PUBLIC_APP_URL=https://abc123.ngrok.io
```

```typescript
// In your UploadThing config
export const { GET, POST } = createRouteHandler({
  router: uploadRouter,
  config: {
    // Use ngrok URL for callbacks in development
    callbackUrl: process.env.NODE_ENV === "development" ? process.env.NEXT_PUBLIC_APP_URL : undefined,
  },
});
```

### **Option 2: Development Mode Bypass**

```typescript
// In your file router
export const uploadRouter = {
  imageUploader: f({
    image: {
      maxFileSize: "32MB",
      maxFileCount: 4,
    },
  })
    .middleware(({ req }) => {
      // In development, skip the callback requirement
      if (process.env.NODE_ENV === "development") {
        return { userId: "dev-user", skipCallback: true };
      }
      return { userId: getUserId(req) };
    })
    .onUploadComplete(async ({ metadata, file }) => {
      // This won't be called in development with skipCallback
      if (metadata.skipCallback) {
        console.log("Development mode: Skipping callback");
        return;
      }

      // Normal production flow
      console.log("Upload complete for userId:", metadata.userId);
      console.log("file url", file.ufsUrl);
    }),
} satisfies FileRouter;
```

### **Option 3: Polling Instead of Webhooks**

```typescript
// Client-side polling for completion
const { startUpload } = useUploadThing("imageUploader", {
  onClientUploadComplete: async (res) => {
    // In development, poll for completion
    if (process.env.NODE_ENV === "development") {
      await pollForCompletion(res[0].key);
    } else {
      // Normal flow in production
      console.log("Upload completed:", res);
    }
  },
});

const pollForCompletion = async (fileKey: string) => {
  const maxAttempts = 30;
  let attempts = 0;

  while (attempts < maxAttempts) {
    try {
      const response = await fetch(`/api/upload/status/${fileKey}`);
      const data = await response.json();

      if (data.status === "completed") {
        console.log("Upload completed via polling:", data);
        return;
      }

      await new Promise((resolve) => setTimeout(resolve, 1000)); // Wait 1 second
      attempts++;
    } catch (error) {
      console.error("Polling error:", error);
      attempts++;
    }
  }

  console.error("Upload completion polling timed out");
};
```

### **Option 4: Use UploadThing's Development Mode**

```typescript
// UploadThing has a development mode that handles this
export const { GET, POST } = createRouteHandler({
  router: uploadRouter,
  config: {
    logLevel: "Debug",
    // This enables development mode with localhost support
    isDev: process.env.NODE_ENV === "development",
  },
});
```

## How Your Current System Compares

### **Your Current Approach:**

```typescript
// Your system uploads to your server first
const formData = new FormData();
formData.append("file", file);

const response = await fetch("/api/memories", {
  method: "POST",
  body: formData, // File goes through your server
});
```

### **UploadThing's Approach:**

```typescript
// UploadThing uploads directly to storage
const { startUpload } = useUploadThing("imageUploader");

await startUpload([file]); // File goes directly to storage
// Your server gets called via webhook after upload completes
```

## The Trade-offs

### **UploadThing's Approach:**

- ✅ **Faster uploads** - Direct to storage
- ✅ **Less server load** - Files don't go through your server
- ✅ **Better for large files** - No server memory limits
- ❌ **Localhost complexity** - Webhook callbacks don't work locally
- ❌ **Less control** - Can't process files before storage

### **Your Current Approach:**

- ✅ **Simple localhost development** - Everything goes through your server
- ✅ **Full control** - Can process files before storage
- ✅ **No callback issues** - Direct API responses
- ❌ **Slower uploads** - Files go through your server first
- ❌ **Server load** - Large files consume server memory

## Recommendations

### **For Your Use Case:**

1. **Keep your current system for ICP users** - It works well and gives you full control
2. **Use UploadThing for standard blob users** - With ngrok for local development
3. **Hybrid approach** - Route based on user preference

### **Development Setup:**

```bash
# Add to your package.json scripts
{
  "scripts": {
    "dev:ngrok": "ngrok http 3000 & npm run dev",
    "dev": "next dev"
  }
}
```

### **Environment Variables:**

```bash
# .env.local
NEXT_PUBLIC_APP_URL=https://your-ngrok-url.ngrok.io
UPLOADTHING_TOKEN=your_token_here
```

## Alternative: Use UploadThing's Client-Only Mode

UploadThing also supports a mode where the completion callback is optional:

```typescript
const { startUpload } = useUploadThing("imageUploader", {
  onClientUploadComplete: (res) => {
    // Handle completion on client side
    console.log("Upload completed:", res);
    // You can call your own API here if needed
    fetch("/api/memories", {
      method: "POST",
      body: JSON.stringify({
        fileUrl: res[0].url,
        fileName: res[0].name,
        // ... other metadata
      }),
    });
  },
});
```

This way, you don't rely on the webhook callback and can handle completion entirely on the client side, which works perfectly in localhost development.

## Conclusion

The localhost callback issue is a real limitation of UploadThing's architecture, but there are several workarounds. For your specific needs with ICP integration, the hybrid approach (keeping your current system for ICP users, using UploadThing for standard users) might be the best solution, with ngrok for local development when testing UploadThing integration.
