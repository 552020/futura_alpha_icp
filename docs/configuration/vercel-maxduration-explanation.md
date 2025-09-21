# Vercel maxDuration Configuration Explained

## What is maxDuration?

`maxDuration` is a Vercel configuration setting that controls how long a serverless function can run before being terminated. It's specified in the `vercel.json` file and applies to API routes and serverless functions.

## Configuration Format

```json
{
  "functions": {
    "src/app/api/memories/route.ts": {
      "maxDuration": 60
    }
  }
}
```

## Limits by Plan

| Plan           | Maximum Duration         | Notes          |
| -------------- | ------------------------ | -------------- |
| **Hobby**      | 10 seconds               | Free tier      |
| **Pro**        | 60 seconds               | Paid tier      |
| **Enterprise** | 900 seconds (15 minutes) | Custom pricing |

## Why We Need This Setting

### The Problem

Our `/api/memories` endpoint processes file uploads, which can take time for:

- File validation
- Storage upload (S3/Vercel Blob)
- Database operations
- Image processing (thumbnails, resizing)

### The Solution

Setting `maxDuration: 60` allows our function to run for up to 60 seconds, giving enough time for:

- Single file uploads
- Small folder uploads
- Processing operations

## Current Configuration

**File**: `src/nextjs/vercel.json`

```json
{
  "functions": {
    "src/app/api/memories/route.ts": {
      "maxDuration": 60
    }
  }
}
```

## Important Notes

### 1. **Still Not Enough for Large Uploads**

Even with 60 seconds, large folder uploads (500+ files) will likely timeout. This is why we need the "grant → direct upload → confirm" flow.

### 2. **Platform Limits Still Apply**

- **Request body size**: Still limited to 4.5MB
- **Memory**: Limited to 1024MB
- **Concurrent executions**: Limited by plan

### 3. **Cost Implications**

- Longer durations = higher costs
- Functions are billed by execution time
- 60 seconds is the sweet spot for most operations

## Error Messages

### Too High Duration

```
The value for maxDuration must be between 1 second and 60 seconds
```

**Fix**: Reduce to 60 seconds or less

### Function Timeout

```
Function execution timed out after X seconds
```

**Fix**: Optimize code or implement direct upload flow

## Best Practices

### 1. **Set Appropriate Duration**

- **Quick operations**: 10-30 seconds
- **File processing**: 60 seconds (max)
- **Long operations**: Use background jobs

### 2. **Optimize Function Performance**

- Use streaming for large files
- Implement direct uploads for large payloads
- Cache expensive operations
- Use database connection pooling

### 3. **Monitor Function Performance**

- Check Vercel dashboard for execution times
- Set up alerts for timeouts
- Monitor error rates

## Alternative Approaches

### For Large Uploads

Instead of increasing `maxDuration`, implement:

1. **Direct Upload Flow**

   ```
   Browser → /api/memories/grant → Presigned URL → Direct Upload → /api/memories/complete
   ```

2. **Chunked Upload**

   ```
   Browser → Split files → Upload chunks → /api/memories/assemble
   ```

3. **Background Processing**
   ```
   Browser → Queue job → /api/memories/status → Poll for completion
   ```

## Troubleshooting

### Function Times Out

1. Check `maxDuration` setting
2. Optimize database queries
3. Use streaming for file operations
4. Implement direct uploads

### Deployment Fails

1. Verify `maxDuration` is ≤ 60 seconds
2. Check Vercel plan limits
3. Ensure proper JSON syntax in `vercel.json`

### Performance Issues

1. Monitor execution times in Vercel dashboard
2. Profile function performance
3. Consider caching strategies
4. Implement connection pooling

## Related Documentation

- [Vercel Functions Documentation](https://vercel.com/docs/functions)
- [Vercel Limits](https://vercel.com/docs/limits)
- [Serverless Function Best Practices](https://vercel.com/docs/functions/serverless-functions)

---

**Last Updated**: 2024-12-19  
**Status**: ✅ **ACTIVE** - Configuration in use
