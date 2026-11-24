# Futura

<img width="1142" height="216" alt="Futura Banner" src="https://github.com/user-attachments/assets/f8a93dd2-8dec-41b8-8bf4-f9e378f0a671" />

Futura combines **ICP** and **blockchain technologies** to safeguard the most meaningful memories of a person for the long term, ensuring they can be passed down across generations.

---

## Status (November 2026)

- **Mainnet:** backend canister `izhgj-eiaaa-aaaaj-a2f7q-cai` is live. The fixes of November 2026 (memory list paging and tokens, gallery visibility, gallery items, Neon binding, inline asset links) are in `main` but **not yet deployed** to mainnet.
- **Tests:** 261 + 2 Rust unit tests and 19 of 19 backend integration scripts pass on a local network.
- **Tooling:** dfx is pinned to 0.31.0 in `dfx.json` (0.32.0's local network fails to start). dfx is deprecated in favour of `icp-cli`; the move is still to do.
- **Analysis:** where the project stands and the state of ICP and blockchain are on the desk of the web app: [stand](https://github.com/552020/futura_alpha_nextjs/blob/main/docs/desk/stand.md), [ICP and blockchain](https://github.com/552020/futura_alpha_nextjs/blob/main/docs/desk/icp-and-blockchain.md).

## How it is built

![Futura running: who talks to whom](https://raw.githubusercontent.com/552020/futura_alpha_nextjs/main/public/diagrams/futura-running.png)

Step by step, from the same sketch generator: [signing in with Internet Identity](https://raw.githubusercontent.com/552020/futura_alpha_nextjs/main/public/diagrams/signin-ii-steps.png), [uploading to ICP](https://raw.githubusercontent.com/552020/futura_alpha_nextjs/main/public/diagrams/upload-icp-steps.png), [the local test setup](https://raw.githubusercontent.com/552020/futura_alpha_nextjs/main/public/diagrams/local-test-setup-running.png).

The web app (Next.js, in [futura_alpha_nextjs](https://github.com/552020/futura_alpha_nextjs)) keeps memories on a Web2 side (Postgres and cloud files) by default. People can choose to keep them in their own capsule in this repository's backend canister instead. The browser writes there directly with the person's Internet Identity key; the web app's server only checks sign-in proofs. The plain-language explanation: [architecture](https://github.com/552020/futura_alpha_nextjs/blob/main/docs/desk/architecture.md).

## Current Offering

At this stage, users can choose between **Web2** and **Web3 storage** options.  
Our initial go-to-market strategy focuses on **wedding photography**:

1. We facilitate the transfer of ownership from the photographer to the newlyweds.
2. Newlyweds are then offered the option to **store their memories forever**.
3. From there, they can expand to preserving other significant data they wish to pass on to future generations—such as autobiographies, personal documents, and cherished memories.

We’ve recently partnered with a professional photographer [Salih, our Partner](https://www.instagram.com/salih_kurbet_films/) and are eager to gather feedback from clients and measure conversion rates over the coming months.

---

## Long-Term Vision

While weddings are our starting point, Futura’s mission goes beyond photography.  
We are building a future of **digital inheritance**, ensuring your legacy endures:

- Your descendants will inherit your **digital archive**—photos, texts, documents, and more.
- They will be able to **interact with your data** through an AI-driven project called **Transcendance**, enabling conversations with the digital reflection of your life.

---

## Features

### 🔐 **Dual Web2/Web3 Authentication**

Futura implements a dual authentication system leveraging Authjs and the Internet Identity, allowing ICP users to safely use the Internet Identity to signin/signup in the Web2 app.

- **Dual Authentication**: Users can sign in with both traditional OAuth providers (Google) and Web3 identity (Internet Identity)
- **Co-Authentication Flow**: Users can link their Internet Identity to existing Web2 accounts, enabling seamless access to both storage systems
- **Session Synchronization**: NextAuth.js manages unified sessions that work across both authentication methods
- **Principal Verification**: Cryptographic nonce-based verification ensures users control their claimed Internet Identity principals

#### **Authentication Flows**

**Flow 1: Traditional Web2 Sign-in (Google)**

```
User → Google OAuth → NextAuth.js → JWT Session → Web2 Storage Access
```

**Flow 2: Internet Identity Standalone Sign-in**

```
User → Internet Identity → Nonce Challenge → Principal Verification → NextAuth.js → JWT Session → Web3 Storage Access
```

**Flow 3: Co-Authentication (Google + Internet Identity)**

```
User → Google OAuth → NextAuth.js Session → Link Internet Identity → Co-Authentication Enabled → Access to Both Storage Systems
```

#### **Technical Challenges Solved**

**1. Internet Identity Integration in Web2 Apps**

- **Challenge**: Internet Identity is designed for Web3 dApps, not traditional Web2 applications
- **Solution**: Custom NextAuth.js provider that handles II authentication flow
- **Implementation**: Direct canister calls for nonce verification, bypassing traditional OAuth flows

**2. Session Management Across Systems**

- **Challenge**: Web2 sessions (JWT) vs Web3 sessions (ICP agent) are fundamentally different
- **Solution**: Unified session management through NextAuth.js with custom JWT claims
- **Implementation**: Store both Web2 user data and Web3 principal information in single JWT token

**3. Principal Verification Security**

- **Challenge**: Ensuring users actually control their claimed Internet Identity principals
- **Solution**: Cryptographic nonce-based challenge-response verification
- **Implementation**: Server-side nonce generation, client-side signing, canister verification

**4. Cross-Platform Compatibility**

- **Challenge**: Internet Identity works differently across browsers and devices
- **Solution**: Robust error handling and fallback mechanisms
- **Implementation**: Multiple authentication paths with graceful degradation

**5. Session Synchronization**

- **Challenge**: Keeping Web2 and Web3 sessions in sync when users switch between them
- **Solution**: Real-time session updates with NextAuth.js `update()` function
- **Implementation**: Automatic session refresh when II authentication state changes

## Technology Approach

Futura uses ICP for what a public network does well:

- **Ownership that does not depend on us:** each person's capsule is written with their own key.
- **Rules for who may see what, and when:** access entries with conditions (scheduled, expiring, and event-based such as "after death", the last one not implemented yet).
- **Passkey login** with Internet Identity.

It is not meant to be the only copy of anyone's files: in 2026 storage on ICP costs roughly 50 times Amazon S3 per GB (details and sources on the desk page linked above).

Our active canister:

```json
"ic": "izhgj-eiaaa-aaaaj-a2f7q-cai"
```

<img width="1030" height="680" alt="Futura Demo Screenshot" src="https://github.com/user-attachments/assets/ce6ca146-34b8-446c-a42e-0004c7ac8087" />

## Clone with submodules

```bash
git clone --recursive https://github.com/552020/futura_alpha_icp.git
```

## Local Development

Verified on Ubuntu 24.04 (WSL2) in November 2026.

1. **Toolchain** (once):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add wasm32-unknown-unknown
   sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"   # dfxvm; dfx.json pins 0.31.0
   ```

2. **Local network and canisters:**

   ```bash
   dfx start --clean --background
   dfx deploy backend && dfx deploy canister_factory && dfx deploy internet_identity
   ```

   On a clean network, in this order, the IDs are: backend `uxrrr-q7777-77774-qaaaq-cai`, canister_factory `uzt4z-lp777-77774-qaabq-cai`, internet_identity `umunu-kh777-77774-qaaca-cai`. Internet Identity is the dev build of release-2026-02-28 with the registration captcha disabled (`init_arg` in `dfx.json`), so browser tests can create identities.

3. **Web app against the local network:** see [e2e/README.md](https://github.com/552020/futura_alpha_nextjs/blob/main/e2e/README.md) in the web app (Node 24, pnpm). `src/nextjs` is a submodule pointing at that repository; after `git submodule update --init src/nextjs`, the backend test scripts use its generated bindings.

`scripts/deploy-local.sh` does steps 2 and also regenerates the `.did` file and the bindings; it needs `generate-did` and `candid-extractor` (`cargo install generate-did candid-extractor`).

## Testing

We've got plenty of tests to keep the codebase together. Our testing setup covers everything from unit tests to integration tests, making sure everything works as expected.

### Frontend Testing (Next.js)

Lives in the web app repository ([futura_alpha_nextjs](https://github.com/552020/futura_alpha_nextjs)): `pnpm test:run` for unit and integration tests, and Playwright for the browser tests, including the Internet Identity flow against a local network ([e2e/README.md](https://github.com/552020/futura_alpha_nextjs/blob/main/e2e/README.md)).

### Backend Testing (ICP)

**Rust unit tests** (no network needed): 261 backend + 2 canister_factory tests.

```bash
cargo test -p backend -p canister_factory
```

**Integration scripts** against the local network: `tests/backend/`, 19 scripts listed in `tests/backend/test_registry.sh`.

```bash
cd tests/backend
npm install                      # once, for the .mjs upload tests
./run_all_tests.sh               # stops at the first failure; --start-from N to continue
```

**PocketIC tests**: `tests/pocket-ic/` (not run in November 2026).

## Deployed Canisters

### Backend Canister (ICP Mainnet)

- **Canister ID**: `izhgj-eiaaa-aaaaj-a2f7q-cai`
- **Canister URL**: https://izhgj-eiaaa-aaaaj-a2f7q-cai.ic0.app
- **Candid Interface**: https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=izhgj-eiaaa-aaaaj-a2f7q-cai

## License

MIT, see [LICENSE](LICENSE).
