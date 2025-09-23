# Futura

<img width="1142" height="216" alt="Futura Banner" src="https://github.com/user-attachments/assets/f8a93dd2-8dec-41b8-8bf4-f9e378f0a671" />

Futura combines **ICP** and **blockchain technologies** to safeguard the most meaningful memories of a person for the long term, ensuring they can be passed down across generations.

---

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

## Technology Approach
Futura leverages **ICP’s storage innovations** and the security of decentralized infrastructure:

- [Dom](https://x.com/dominic_w/status/1955447139347337491) (ICP) has announced unprecedented reductions in storage costs, supporting projects like **Caffeine**. We build upon this strategy to make **long-term, highly accessible storage** possible.  
- Decentralization ensures the **longevity and security** of hosted files.  
- We focus on **data ownership and ownership transfer**, particularly in the delivery of digital files.  

Our active canister:  
```json
"ic": "izhgj-eiaaa-aaaaj-a2f7q-cai"
```


<img width="1030" height="680" alt="Futura Demo Screenshot" src="https://github.com/user-attachments/assets/ce6ca146-34b8-446c-a42e-0004c7ac8087" /> 


## Clone with submodules

```bash
git clone --recursive https://github.com/552020/futura_alpha_icp.git
```

### Local dev quickstart

```bash
dfx start --background
./scripts/deploy-local.sh
# Optional: run Next.js locally
cd src/nextjs && npm run dev
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
