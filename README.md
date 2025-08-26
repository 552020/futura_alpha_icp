# FUTURA alpha

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
   chmod +x scripts/deploy.sh  # Only needed first time
   ./scripts/deploy.sh
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
