Here’s what I found out about **utServer** (the “unofficial UploadThing self-host”) plus deeper info on **Tus + tusd + Uppy** and some other options from your “#10 alternatives” list — pros, cons, how they might map to your ICP / custom storage / hybrid needs.

---

## utServer (brrock/utserver)

This is a community project: “The unofficial way to self host uploadthing. Fully compatible with uploadthing.” ([GitHub][1])

**What it does:**

- It aims to mimic the UploadThing backend enough so that the official UploadThing client SDKs can use it “out of the box”. ([GitHub][1])
- Written in **Hono** (a fast, lightweight TypeScript framework). ([GitHub][1])
- Uses a database (currently SQLite) for metadata/storage of upload state. ([GitHub][1])
- Has some environment variables to configure base URLs, API token, etc., so that the UploadThing SDKs don’t know the difference (i.e. they think they are communicating with “official” endpoints). ([GitHub][1])

\*\*What it **doesn't (or at least not yet) do / limitations:**

- It likely does **not** have all features of the official UploadThing backend (dashboard, billing, usage metering, advanced storage scaling).
- The storage adapter roadmap includes “S3 storage adapter” and “Vercel blob storage adapter” — meaning currently storage options might be limited. ([GitHub][1])
- It is “unofficial”, meaning stability, feature coverage, compatibility with future versions of UploadThing, or long-term maintenance is not guaranteed.

**How well it could work for your architecture:**

- Could give you much of the functionality of UploadThing but self-hosted — good for reducing dependency and cost.
- But unless it supports your custom storage providers (ICP canisters, Neon DB etc.) or you extend it, you’ll still need custom work.
- Because UT client SDKs expect certain endpoints & behaviors, using utServer lets you leverage the SDKs, but for ICP or other exotic backends, you might need to extend utServer or override parts of it.

---

## Tus + tusd + Uppy

Here’s a breakdown — what they are, how they work, how they map to your requirements, plus strengths/constraints.

| Component | What it is / role                                                                                                                                                          | How it works (high level)                                                                                                                                                                                                                     |
| --------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **tus**   | An **open protocol** for resumable uploads over HTTP. Designed so uploads can be paused, resumed, survive client crashes, network drops. ([Tus][2])                        | Client (browser, mobile) sends a `POST` to initiate, then `PATCH` requests with byte ranges; server keeps track of what's uploaded so far; supports metadata; can use extensions. ([Tus][3])                                                  |
| **tusd**  | The official **Go server implementation** of tus. Open source. Support for various storage backends (filesystem, S3, GCS, S3-compatible) and hooks. ([GitHub][4])          | You run tusd in your infra; configure storage, upload URLs, endpoints; it exposes HTTP API endpoints. After upload completes, you can trigger hooks or have your backend poll/observe uploaded files to do processing, metadata storage, etc. |
| **Uppy**  | Frontend uploader toolkit / UI. Provides dashboards, progress, hooks, supports plugins for different upload sources, resumable upload protocols (like tus). ([uppy.io][5]) | On client side, you integrate Uppy components; configure it with the tus plugin pointing at your tusd server’s endpoint; Uppy handles UI, retry/resume, metadata passing etc.                                                                 |

---

## Pros & Trade-offs of Tus/tusd/Uppy

**Pros:**

- Full control: you host the server and storage; you decide which storage backend, where, what policies.
- Resumability: large files can survive interruptions. Useful for poor network/mobile etc.
- Multiple backends: S3, GCS, local disk, or any S3-compatible (MinIO, etc.). ([tus.github.io][6])
- Hooks / events are available (e.g. tusd supports `--hooks-http`, `--hooks-dir`) so you can trigger your own post-processing, notifications etc. ([GitHub][4])
- Proven in production by many folks. Lots of community experience / examples.

**Trade-offs / limitations:**

- You have to manage more infra: scaling, storage, backups, servers.
- Custom storage providers that are not S3/GCS/S3-compatible may require you to write your own DataStore (if the protocol allows it) or adapt. For example, adding ICP canister storage will likely require a custom adapter or shim.
- Some tus implementations (like tusd with S3) have constraints: e.g., S3 has minimum part sizes for multipart uploads, eventual consistency, etc. (These sometimes affect resumability semantics or latency of “visibility” of upload completion). ([Tus][7])
- Setting up security (auth, upload quotas, metadata, temp users) is on you.

---

## Other Options from “#10 Open-Source Alternatives”

Here are some of the ones I mentioned earlier, with more detail, especially wrt your needs:

| Alternative                                                                                                                                                                 | Description / maturity                                                                       | How it handles / could handle your custom needs (ICP, staging users, processing variants)                                                                                                                                                 |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Supabase Storage**                                                                                                                                                        | OSS with frontend SDKs, you can self-host parts; it's basically a Postgres + storage system. | You can store files, set up presigned URLs, and trigger edge functions or serverless functions for processing. You’d need to build in ICP flows or custom storage. Good if your storage needs are standard (blob-style) or S3-compatible. |
| **MinIO**                                                                                                                                                                   | S3-compatible object storage, you self-host.                                                 | You host MinIO; combine with presigned URLs or your backend endpoints; processing via worker queue etc. Full control. Can integrate with ICP for storage (if ICP can be exposed as an object store) or wrap writes to canister.           |
| **tus / tusd / Uppy** (just discussed)                                                                                                                                      | Very mature for resumability; many examples.                                                 | Might require custom adapter or “DataStore” implementation for ICP; you can run tusd and have your backend pick up the files and move to ICP or also mirror.                                                                              |
| **DIY Flow**                                                                                                                                                                | Your current custom hooks + storage + processing.                                            | You already have this; you might reuse some UI or logic. If you adopt parts of UploadThing or tusd, you can keep this for ICP users.                                                                                                      |
| **Frameworks / Upload Microservices** (like using Express / Actix / Rocket or what­ever in Rust) + presigned uploads + post-processing pipelines (image Magic / Sharp etc.) | Building from components.                                                                    | Gives you maximum control. More work. Can support ICP, Neon, any storage. If performance/scale allowed, this is robust.                                                                                                                   |

---

## How utServer + Tus might combine with your architecture

You might consider a hybrid design:

- For “standard” users (non-ICP), use **utServer** (self-hosting UT) so you can use the UploadThing SDK but host the backend yourself. If utServer supports S3 / Vercel blob / your preferred blobs, use that.
- For ICP users, use tusd + Uppy + custom DataStore that writes into ICP canister (or write via your backend after upload). Process image variants after completion.
- Common metadata layer (in your Neon DB) to track file id / provider / status / URLs / variants.

---

If you like, I can pull together a **comparison table** between utServer vs tusd vs your current system vs full UT SaaS, specifically for your requirements (ICP, image variants, onboarding users, storage flexibility). Do you want that?

[1]: https://github.com/brrock/utserver "GitHub - brrock/utServer: The unofficial way to self host uploadthing. Fully compatiable with uploadthing. Written in hono for fast speeds"
[2]: https://tus.io/?utm_source=chatgpt.com "tus - resumable file uploads"
[3]: https://tus.io/faq?utm_source=chatgpt.com "FAQ"
[4]: https://github.com/tusbpr/tusd?utm_source=chatgpt.com "tusbpr/tusd: this is customization tusd"
[5]: https://uppy.io/docs/tus/?utm_source=chatgpt.com "Tus"
[6]: https://tus.github.io/tusd/advanced-topics/usage-package/?utm_source=chatgpt.com "Embedding in Go programs | tusd documentation"
[7]: https://tus.io/blog/2016/03/07/tus-s3-backend?utm_source=chatgpt.com "S3 as a Storage Back-End"
