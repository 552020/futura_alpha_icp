# Bytes to Image Lab

This lab explores how to transform raw bytes from ICP canisters into images that browsers can understand and display.

## The Problem

When you fetch image data from an ICP canister, you get raw bytes (a `[nat8]` array in Candid). These are just numbers like `[255, 216, 255, 224, 0, 16, 74, 70, 73, 70, ...]` that represent the binary data of an image file. Browsers can't directly display these raw numbers as images.

## The Solution

We need to convert these raw bytes into a format that browsers understand. There are several approaches:

### 1. Base64 Data URLs

Convert the raw bytes to Base64 encoding and create a data URL:

```javascript
// Raw bytes from ICP: [255, 216, 255, 224, ...]
const base64 = btoa(String.fromCharCode(...bytes));
const dataUrl = `data:image/jpeg;base64,${base64}`;
// Result: "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQ..."
```

### 2. Blob URLs

Create a Blob from the raw bytes and generate a URL:

```javascript
const blob = new Blob([bytes], { type: "image/jpeg" });
const blobUrl = URL.createObjectURL(blob);
// Result: "blob:http://localhost:3000/12345678-1234-1234-1234-123456789abc"
```

## Files

- `index.html` - Interactive demo showing both conversion methods
- `README.md` - This documentation

## How to Use

### Important: Running the Lab

**⚠️ CORS Issue**: You cannot open the HTML file directly (`file://`) because browsers block `fetch()` requests from `file://` URLs for security reasons. You'll see this error:

```
Access to fetch at 'file:///...' from origin 'null' has been blocked by CORS policy
```

**Solution**: Serve the files through a web server:

#### Option 1: Node.js Serve

```bash
npx serve .
```

#### Option 2: NPM/PNPM with http-server

```bash
# Install globally
npm install -g http-server
# or
pnpm add -g http-server

# Run
http-server
# or
pnpm http-server
```

#### Option 3: VS Code Live Server

Use the "Live Server" extension in VS Code.

### Steps

1. **Start a web server** (see above)
2. **Open the HTML file** in a web browser via HTTP
3. **Observe raw browser behavior** with different data types
4. **See the transformation** from raw bytes to displayable images
5. **Compare blob sizes** between hardcoded and fetched data

## Real Implementation

In a real application, you would:

1. **Import your IDL factory**:

   ```javascript
   import { idlFactory } from "./your_canister.did.js";
   ```

2. **Create the agent and actor**:

   ```javascript
   import { HttpAgent, Actor } from "@dfinity/agent";

   const agent = new HttpAgent({ host: "https://icp0.io" });
   const actor = Actor.createActor(idlFactory, { agent, canisterId });
   ```

3. **Call your canister method**:

   ```javascript
   const bytes = await actor.get_photo("wedding.jpg");
   ```

4. **Convert to displayable format**:
   ```javascript
   const base64 = btoa(String.fromCharCode(...bytes));
   const dataUrl = `data:image/jpeg;base64,${base64}`;
   ```

## Image Type Detection

The lab includes automatic image type detection based on file signatures:

- **JPEG**: `0xFF 0xD8 0xFF`
- **PNG**: `0x89 0x50 0x4E 0x47`
- **GIF**: `0x47 0x49 0x46`
- **BMP**: `0x42 0x4D`
- **WebP**: `0x52 0x49 0x46 0x46`

## Key Concepts

### Raw Bytes

- ICP canisters return image data as `[nat8]` arrays
- These are just numbers representing binary data
- Browsers can't display raw numbers as images

### Base64 Encoding

- Converts binary data to ASCII text
- Safe for URLs and data transmission
- Increases size by ~33%
- Creates data URLs like `data:image/jpeg;base64,...`

### Blob URLs

- Creates temporary URLs for binary data
- More memory efficient for large images
- URLs are automatically cleaned up
- Better for dynamic content

### Data URLs vs Blob URLs

| Feature      | Data URLs               | Blob URLs                        |
| ------------ | ----------------------- | -------------------------------- |
| **Size**     | Larger (Base64 encoded) | Smaller (binary)                 |
| **Memory**   | Stored in URL string    | Stored in memory                 |
| **Cleanup**  | Automatic               | Manual (`URL.revokeObjectURL()`) |
| **Caching**  | Cached by browser       | Not cached                       |
| **Use case** | Small images, inline    | Large images, dynamic            |

## Browser Compatibility

Both methods work in all modern browsers:

- ✅ Chrome 4+
- ✅ Firefox 3.6+
- ✅ Safari 3.1+
- ✅ Edge 12+

## Performance Considerations

- **Small images** (< 1MB): Use data URLs
- **Large images** (> 1MB): Use Blob URLs
- **Many images**: Use Blob URLs to avoid memory issues
- **Static images**: Consider pre-converting to Base64

## Security Notes

- Data URLs can be large and may hit URL length limits
- Blob URLs are scoped to the origin
- Always validate image data before display
- Consider content security policies for data URLs

## Example Output

When you run the lab, you'll see:

```
✅ Successfully fetched 1234 bytes from ICP canister!

📊 Image Analysis:
• Detected type: image/jpeg
• Size: 1234 bytes
• First 20 bytes (hex): ff d8 ff e0 00 10 4a 46 49 46 00 01 01 01 00 48 00 48 00 00

🔍 Raw bytes (first 50):
[255, 216, 255, 224, 0, 16, 74, 70, 73, 70, 0, 1, 1, 1, 0, 72, 0, 72, 0, 0, ...]
```

## Next Steps

1. **Integrate with your canister**: Replace the demo code with real ICP calls
2. **Add error handling**: Handle network errors, invalid images, etc.
3. **Optimize performance**: Use Blob URLs for large images
4. **Add image processing**: Resize, crop, or apply filters
5. **Implement caching**: Store converted images for reuse

## Related Concepts

- **Candid types**: Understanding `[nat8]` arrays
- **ICP agents**: HttpAgent and Actor patterns
- **Binary data**: Working with raw bytes in JavaScript
- **Image formats**: JPEG, PNG, GIF, WebP signatures
- **Browser APIs**: Blob, URL, btoa/atob functions

This lab provides a foundation for working with binary image data from ICP canisters in web applications.
