// import { NextRequest, NextResponse } from 'next/server';

// function idem() {
//   return typeof crypto !== 'undefined' && 'randomUUID' in crypto
//     ? crypto.randomUUID()
//     : `${Date.now()}-${Math.random().toString(36).slice(2)}`;
// }

// export async function POST(request: NextRequest) {
//   try {
//     const body = await request.json().catch(() => ({}));
//     const blobHosting = body?.hostingPreferences?.blobHosting as
//       | 's3'
//       | 'vercel_blob'
//       | 'icp'
//       | 'arweave'
//       | 'ipfs'
//       | undefined;
//     // const databaseHosting = body?.hostingPreferences?.databaseHosting as 'neon' | 'icp' | undefined; // Not used yet

//     // The intent route should return both database and blob storage info
//     // Since this is a Vercel route, database is always Neon
//     const database = 'neon'; // Always Neon for Vercel backend
//     const blob_storage = blobHosting || 's3'; // User's blob preference or default to S3

//     const ttl_seconds = 600; // 10 minutes
//     const expires_at = new Date(Date.now() + ttl_seconds * 1000).toISOString();

//     const payload: {
//       uploadStorage: {
//         database: 'neon' | 'icp';
//         blob_storage: 's3' | 'vercel_blob' | 'icp' | 'arweave' | 'ipfs';
//         idem: string;
//         expires_at: string;
//         ttl_seconds: number;
//         limits: { inline_max: number; chunk_size: number; max_chunks: number };
//         icp?: { canister_id: string; network?: string };
//       };
//     } = {
//       uploadStorage: {
//         database,
//         blob_storage,
//         idem: idem(),
//         expires_at,
//         ttl_seconds,
//         limits: { inline_max: 32 * 1024, chunk_size: 64 * 1024, max_chunks: 512 },
//       },
//     };

//     if (blob_storage === 'icp') {
//       payload.uploadStorage.icp = {
//         canister_id: process.env.NEXT_PUBLIC_ICP_CANISTER_ID ?? 'ryjl3-tyaaa-aaaaa-aaaba-cai',
//         network: process.env.NEXT_PUBLIC_ICP_NETWORK ?? 'ic',
//       };
//     }

//     return NextResponse.json(payload, { status: 200 });
//   } catch {
//     return NextResponse.json(
//       { error: 'INTERNAL_ERROR', message: 'Failed to create upload storage selection' },
//       { status: 500 }
//     );
//   }
// }
