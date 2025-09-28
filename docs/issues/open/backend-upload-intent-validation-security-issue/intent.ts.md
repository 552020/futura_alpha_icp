// import { useMutation } from '@tanstack/react-query';
// // import type { HostingPreferences } from '@/hooks/use-storage-preferences'; // Not used yet

// export interface UploadStorage {
//   database: 'neon' | 'icp';
//   blob_storage: 's3' | 'vercel_blob' | 'icp' | 'arweave' | 'ipfs';
//   idem: string;
//   expires_at: string; // ISO
//   ttl_seconds: number;
//   limits?: { inline_max: number; chunk_size: number; max_chunks: number };
//   icp?: { canister_id: string; network?: string };
// }

// export interface UploadStorageResponse {
//   uploadStorage: UploadStorage;
// }

// export function isUploadStorageExpired(expires_at: string): boolean {
//   return Date.now() > new Date(expires_at).getTime();
// }

// type IntentVars = {
//   preferred?: 's3' | 'vercel_blob' | 'icp' | 'arweave' | 'ipfs';
//   databaseHosting?: 'neon' | 'icp';
//   backendHosting?: 'vercel' | 'icp';
// };

// /**
//  * Verifies upload intent by calling the appropriate backend
//  * Routes between Vercel and ICP backends based on hosting preferences
//  */
// export async function verifyIntent(vars: IntentVars): Promise<UploadStorageResponse> {
//   const { backendHosting, preferred, databaseHosting } = vars;

//   if (backendHosting === 'icp') {
//     // Call ICP canister directly
//     return await callICPIntent();
//   } else {
//     // Call Vercel intent route
//     return await callVercelIntent({ preferred, databaseHosting });
//   }
// }

// /**
//  * Calls Vercel intent route
//  */
// async function callVercelIntent(vars: {
//   preferred?: string;
//   databaseHosting?: string;
// }): Promise<UploadStorageResponse> {
//   const res = await fetch('/api/upload/intent', {
//     method: 'POST',
//     headers: {
//       'Content-Type': 'application/json',
//       'Idempotency-Key':
//         typeof crypto !== 'undefined' && 'randomUUID' in crypto
//           ? crypto.randomUUID()
//           : `${Date.now()}-${Math.random().toString(36).slice(2)}`,
//     },
//     body: JSON.stringify({
//       hostingPreferences: {
//         blobHosting: vars?.preferred,
//         databaseHosting: vars?.databaseHosting,
//       },
//     }),
//   });

//   if (!res.ok) throw new Error(`HTTP ${res.status}`);
//   return res.json();
// }

// /**
//  * Calls ICP canister directly for intent verification
//  * TODO: Implement ICP canister call
//  */
// async function callICPIntent(): Promise<UploadStorageResponse> {
//   // TODO: Implement ICP canister intent call
//   throw new Error('ICP intent verification not yet implemented');
// }

// /**
//  * React hook for intent verification
//  */
// export function useIntentVerification() {
//   return useMutation<UploadStorageResponse, Error, IntentVars>({
//     mutationFn: verifyIntent,
//   });
// }
