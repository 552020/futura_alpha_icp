// ic-identity.js
import fs from "fs";
import path from "path";
import { execSync } from "child_process";
import { HttpAgent } from "@dfinity/agent";
import { Ed25519KeyIdentity } from "@dfinity/identity";

// Helpers
function pemToDer(pem) {
  const b64 = pem
    .replace(/-----BEGIN [^-]+-----/g, "")
    .replace(/-----END [^-]+-----/g, "")
    .replace(/\s+/g, "");
  return Buffer.from(b64, "base64");
}

function derToPem(der, label) {
  const b64 = der.toString("base64").replace(/(.{64})/g, "$1\n");
  return `-----BEGIN ${label}-----\n${b64}\n-----END ${label}-----\n`;
}

// Try to build identity from PKCS#8 PEM.
// Newer @dfinity/identity provides fromPEM; if not, we extract seed bytes.
function ed25519IdentityFromPem(pem) {
  if (typeof Ed25519KeyIdentity.fromPEM === "function") {
    return Ed25519KeyIdentity.fromPEM(pem);
  }
  // Fallback: minimal PKCS#8 parse for Ed25519
  const der = pemToDer(pem);
  // PKCS#8 privateKey is an OCTET STRING containing another OCTET STRING of 32-byte seed (RFC 8410).
  // This quick-and-dirty parser finds the last 32 bytes.
  // (Safe for standard Ed25519 keys exported by dfx.)
  const SEED_LEN = 32;
  const seed = der.slice(-SEED_LEN);
  if (seed.length !== SEED_LEN) {
    throw new Error("Could not extract Ed25519 seed from PEM DER.");
  }
  return Ed25519KeyIdentity.fromSecretKey(new Uint8Array(seed));
}

export function loadDfxIdentity(explicitName) {
  const home = process.env.HOME || process.env.USERPROFILE;
  const name = explicitName?.trim() || execSync("dfx identity whoami", { encoding: "utf8" }).trim();
  const dir = path.join(home, ".config", "dfx", "identity", name);

  // 1) PEM on disk
  const pemPath = path.join(dir, "identity.pem");
  if (fs.existsSync(pemPath)) {
    const pem = fs.readFileSync(pemPath, "utf8");
    return ed25519IdentityFromPem(pem);
  }

  // 2) Keyring JSON on disk (DER-encoded keys)
  const jsonPath = path.join(dir, "identity.json");
  if (fs.existsSync(jsonPath)) {
    const raw = JSON.parse(fs.readFileSync(jsonPath, "utf8"));
    // Check if this is a keyring identity with DER-encoded keys
    if (raw.der_encoded_secret_key) {
      // dfx stores PKCS#8 (private) and SPKI (public) as base64 DER
      const privDer = Buffer.from(raw.der_encoded_secret_key, "base64");
      // Convert DER to PEM and reuse PEM path
      const pem = derToPem(privDer, "PRIVATE KEY");
      return ed25519IdentityFromPem(pem);
    } else {
      // This is a keyring identity without DER keys, fall through to export
      console.log(`Keyring identity ${name} found, will export via dfx`);
    }
  }

  // 3) Ask dfx to export a PEM (works for keyring identities too)
  const exported = execSync(`dfx identity export ${name}`, { encoding: "utf8" });
  return ed25519IdentityFromPem(exported);
}

export function makeMainnetAgent(identity) {
  // ic host aliases: https://ic0.app or https://icp0.io (both valid)
  return new HttpAgent({
    host: "https://ic0.app",
    identity,
    fetch, // node-fetch or global fetch in Node 18+ / 22+
  });
}
