# Serving Assets Over HTTP on ICP

## Overview

This document explains how to serve assets over HTTP on the Internet Computer Protocol (ICP), covering both public and private asset serving strategies.

## HTTPS Always Works

The boundary node provides every `http_request` with a normal HTTPS interface, ensuring secure communication regardless of certification status.

## Certification: Optional but Powerful

### With Certification

- **Benefits**: Proof + caching
- **Use Case**: Public immutable files
- **Behavior**: Globally verifiable and cacheable
- **Implementation**: Include `ic-certificate` header in response

### Without Certification

- **Benefits**: Per-request processing, no global caching
- **Use Case**: Private or personalized content
- **Behavior**: HTTPS but uncached and per-request
- **Implementation**: Return `HttpResponse` without `ic-certificate` header

## Dual Strategy Implementation

You can serve both public and private assets from the same canister using a routing strategy:

```
/pub/...   → certified assets (immutable, cacheable)
/priv/...  → non-certified route (ACL logic, private)
```

## Private Asset Flow

For serving private assets (`/priv/...`), the flow is:

1. **Browser** → **Boundary Node** → **Your Canister's `http_request`**
2. **Your code** checks identity/permission
3. **You read** the bytes (from stable memory or asset store)
4. **You return** an `HttpResponse` **without** `ic-certificate` header

## Key Points

- ✅ **HTTPS always works** — boundary node provides secure interface
- ✅ **Certification is optional** — choose based on use case
- ✅ **Private assets supported** — use non-certified responses
- ✅ **Same canister strategy** — serve both public and private content
- ✅ **Safe and supported** — this is the recommended approach for private ICP assets

## Security Considerations

- Private routes should implement proper access control logic
- Identity verification should be performed before serving sensitive content
- Consider rate limiting for private endpoints
- Log access attempts for security monitoring

## The Underlying Problem

The Internet Computer Protocol (ICP) provides two fundamentally different ways to access canister data, and understanding this distinction is crucial for building web applications.

### Architecture Overview

Both access methods go through the same infrastructure but with different payload formats:

```
┌─────────────────┐    HTTP Request     ┌──────────────────┐
│   Your Client   │ ──────────────────► │  Boundary Node   │
│                 │                     │                  │
│ (Browser/Agent) │ ◄────────────────── │                  │
└─────────────────┘    HTTP Response    └──────────────────┘
                                               │
                                               │ ICP Protocol
                                               ▼
                                        ┌──────────────────┐
                                        │    Canister      │
                                        │                  │
                                        │ - Raw methods    │
                                        │ - http_request   │
                                        └──────────────────┘
```

**Key Point**: The boundary node is the gateway that:

1. Receives HTTP requests from clients
2. Unpacks the payload (either ICP-standard or HTTP-standard)
3. Makes the appropriate call to the canister
4. Packs the response back into HTTP format
5. Sends it back to the client

### The Special `http_request` Method

On the Internet Computer, `http_request` is considered a "special method" because it has a specific role in the canister interface: it is the entry point that the HTTP Gateway uses to route incoming HTTP requests to your canister. When a browser or HTTP client tries to access your canister as if it were a web server, the gateway translates the HTTP request into a call to your canister's `http_request` method. The method must follow a specific signature and return an HTTP response structure as defined in the HTTP Gateway Protocol Specification.

### 1. The "Raw" Format on the Internet Computer

Every canister on the Internet Computer can expose **methods** (functions) that you call through the ICP protocol.

Example:

```did
service : {
  get_photo : (text asset_id) -> (blob);
}
```

You can call that method from code using `@dfinity/agent`, or from `dfx`:

```bash
dfx canister call mycanister get_photo '("wedding.jpg")'
```

**Important**: You don't call canisters directly. The call is packed in the body of an HTTP request that goes to the boundary node. The boundary node unpacks the payload (which is written in a special ICP standard, not as a web server request) and makes the call to the canister. The canister responds to the boundary node, which packs the content in an HTTP response.

What you get back is just **raw bytes** — whatever the function returns.
It's not "HTTP," it's a **canister call** (a blockchain message) that travels through HTTP transport.

So you could fetch the photo bytes this way and display them manually in a web app:

```ts
const bytes = await actor.get_photo("wedding.jpg");
const blobUrl = URL.createObjectURL(new Blob([new Uint8Array(bytes)], { type: "image/jpeg" }));
img.src = blobUrl;
```

That's "raw format" — direct protocol access, no browser HTTP layer involved.

### 2. The "Over HTTP" Format

The Internet Computer also allows a canister to **act like a web server** through the special method `http_request`.

```did
http_request : (HttpRequest) -> (HttpResponse);
```

When you open `https://<canister-id>.icp0.io/photo.jpg` in your browser:

1. The **browser** sends a normal HTTPS request to a **boundary node**.
2. The **boundary node** calls the canister's `http_request` method.
3. The **canister** returns an `HttpResponse`:

   ```rust
   {
     status_code: 200,
     headers: [("Content-Type", "image/jpeg")],
     body: [ ... bytes of image ... ]
   }
   ```

4. The boundary node wraps that in a normal HTTPS response and sends it back to the browser.

→ From the browser's point of view, it's just a normal web server.
→ From ICP's point of view, it's a canister call disguised as HTTP.

So "over HTTP" means: **same data, but delivered through an HTTPS-compatible interface** that browsers can use directly.

### 3. Why Both Exist

| Access method                        | Who can use it                                    | Typical use                     | Returns                              |
| ------------------------------------ | ------------------------------------------------- | ------------------------------- | ------------------------------------ |
| **Raw call (Candid / ICP protocol)** | Apps using `@dfinity/agent`, CLI, other canisters | programmatic data exchange      | Typed data (blobs, numbers, structs) |
| **HTTP (`http_request`)**            | Any web browser                                   | serving web pages, images, etc. | HTTP responses (HTML, JSON, images)  |

- The **raw call** is like an API for code.
- The **HTTP request** is like a website for humans (and browsers).

Under the hood they both talk to the same canister, but through different entry points.

### 4. Why Not Just Use "Raw" for Everything?

Because browsers don't know the ICP protocol.
They only understand **HTTP/HTTPS**.

So if you want people to access your canister through a web address (`https://...`),
you need to expose an **`http_request`** method.

That's what the **asset canisters** and the **asset library** do:
they implement `http_request` so your images and HTML files can be fetched by browsers.

### 5. The Certification Layer (Optional, on Top of HTTP)

Certification just adds **proofs** to those HTTP responses.
It's a blockchain way of saying:

> "Here's your `/photo.jpg` file, and here's a cryptographic proof that it's exactly what's stored on-chain."

Without certification:

- still HTTPS,
- still safe to fetch,
- but no proof (the boundary node could, in theory, alter it).

With certification:

- proof added,
- can be cached safely by boundary nodes,
- must be public (same file for everyone).

### 6. Summary

| Term                   | What it really means                                                                          |
| ---------------------- | --------------------------------------------------------------------------------------------- |
| **Raw access**         | Calling a canister method (not HTTP). You get bytes directly through the ICP protocol.        |
| **HTTP access**        | Boundary node turns your HTTPS request into a canister `http_request` call. Browser-friendly. |
| **Certified HTTP**     | Same as HTTP, but with a Merkle proof so you can trust and cache it.                          |
| **Non-certified HTTP** | Same as HTTP, but without the proof (used for private/ACL content).                           |

### Architecture Overview

```
         ┌──────────────────────┐
Browser  │ HTTPS (normal web)   │
         │                      │
         ▼                      │
   Boundary Node  ───►  http_request  ───►  Canister code  (serves bytes)
                                 │
                                 ▼
                          Canister state (stored assets)
```

And separately:

```
Your Dapp backend  ─►  get_photo("id")  ─►  Canister  (raw data)
```

Same canister, two different "doors":

- **door 1:** raw protocol (for apps)
- **door 2:** http gateway (for browsers)

---
