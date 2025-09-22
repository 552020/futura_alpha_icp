# FUTURA alpha

Submission for the **ICP WCHL25 National Round**.

## Hackathon submission overview

The core of this project is leveraging ICP and blockchain technologies to preserve memories long term — and, ultimately, aspects of the person themself.

We are working on three verticals:

1. **Digital vault**: share and store valuable memories across generations
2. **Forever wedding album**: permanent, shareable album with optional AI curation
3. **AI‑Self for the aftermath**: legacy messages and guided memory preservation

### 1. Offline validation

We conducted market research across all three verticals to validate product-market fit.

#### Offline validation (market research)

**Digital vault**: we conducted interviews with potential customers (parents, family archivists, people who already keep photo libraries). Main takeaways are: difficulty of reaching target audience, higher age of possible targets, need is not well defined, and the target niche is a spread target group.

**Wedding album**: in this case it's really easy to reach out, the market is big, target is much younger and interviews showed strong interest. We reached out to wedding photographers and planners to test interest in a permanent, easily shareable album with AI curation and guest contributions. Early partners are interested, especially for differentiating their packages.

**AI‑Self**: we ran a quick survey and exploratory calls around memorialization/legacy messages and an "AI companion clone." There's curiosity about leaving messages to loved ones and organizing digital life in one place, but consent, privacy, and tone are critical. Early signal says start with narrow use cases (e.g., guided legacy prompts) rather than a full conversational clone.

### 2. Juno fake‑door landing pages

We prepared three simple landing pages (one per vertical) for an upcoming A/B test to measure interest.

- Source: see the submodule `src/nextjs`, branch `juno`
- Deployed on Juno: [add link here]

The goal is to validate messaging, value prop, and capture early sign‑ups.

### 3. Web2 MVP with ICP integration

We continued working on the web2 MVP. It’s a full‑stack Next.js app, and we integrated an ICP backend to store memories.

- Next.js app (web2) provides most of the needed functionality and UX
- Memory HTTP certification and storage code is scaffolded in the Rust canister but temporarily commented out to keep the current build clean while we iterate
- Internet Identity is included for future sign‑in flows; frontend canister deployment is disabled for now to simplify local deploys

### Local dev quickstart

```bash
dfx start --background
./scripts/deploy-local.sh
# Optional: run Next.js locally
cd src/nextjs && npm run dev
```

## Clone with submodules

```bash
git clone --recursive https://github.com/552020/futura_alpha_icp.git
```

## Local Development

To test and develop locally:

0. **Setup environment**:

   ```bash
   # Copy the example file and fill in your values
   cp src/nextjs/.env.local.example src/nextjs/.env.local
   # Edit src/nextjs/.env.local with your actual credentials
   ```

1. **Start DFX** (Internet Computer local replica):

   ```bash
   dfx start --clean --background
   ```

2. **Deploy canisters**:

   ```bash
   # Install required tools for the deploy script
   cargo install generate-did
   cargo install ic-cdk-optimizer --locked
   cargo install candid-extractor --locked

   chmod +x scripts/deploy-local.sh  # Only needed first time
   ./scripts/deploy-local.sh
   ```

3. **Start Next.js development server**:
   ```bash
   cd src/nextjs
   npm run dev
   ```

## Deployed Canisters

### Backend Canister (ICP Mainnet)

- **Canister ID**: `izhgj-eiaaa-aaaaj-a2f7q-cai`
- **Canister URL**: https://izhgj-eiaaa-aaaaj-a2f7q-cai.ic0.app
- **Candid Interface**: https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=izhgj-eiaaa-aaaaj-a2f7q-cai
