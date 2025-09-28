import { NextRequest, NextResponse } from 'next/server';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { app_memory_id, database, blob_storage, idem, checksum_sha256, size, remote_id, session_id } = body ?? {};

    if (!app_memory_id || !database || !blob_storage || !idem) {
      return NextResponse.json(
        {
          error: 'VALIDATION_ERROR',
          message: 'Missing required fields',
          data: {
            field_errors: {
              app_memory_id: app_memory_id ? [] : ['required'],
              database: database ? [] : ['required'],
              blob_storage: blob_storage ? [] : ['required'],
              idem: idem ? [] : ['required'],
            },
          },
        },
        { status: 422 }
      );
    }

    // Mock verification for Vercel backend
    // This route only handles Vercel backend verification
    // ICP verification is handled by frontend calling ICP canister directly

    // 1. Verify memory exists in database (Neon)
    const memoryExists = await verifyMemoryInDatabase(app_memory_id);

    // 2. Verify blob exists in storage (S3/Vercel Blob/Arweave/IPFS)
    const blobExists = await verifyBlobInStorage(remote_id);

    // Create storage edge records for the new structure
    const storage_edges = [
      // Metadata storage edge
      {
        id: `edge_metadata_${Date.now()}`,
        memoryId: app_memory_id,
        memoryType: 'image', // TODO: Get from request or memory lookup
        artifact: 'metadata',
        locationMetadata: database, // 'neon' | 'icp'
        locationAsset: null,
        present: memoryExists,
        locationUrl: null,
        contentHash: null,
        sizeBytes: null,
        syncState: 'idle',
        verified_at: new Date().toISOString(),
      },
      // Asset storage edge
      {
        id: `edge_asset_${Date.now()}`,
        memoryId: app_memory_id,
        memoryType: 'image', // TODO: Get from request or memory lookup
        artifact: 'asset',
        locationMetadata: null,
        locationAsset: blob_storage, // 's3' | 'vercel_blob' | 'icp' | 'arweave' | 'ipfs'
        present: blobExists,
        locationUrl: remote_id ?? session_id ?? 'mock-remote',
        contentHash: checksum_sha256 ?? null,
        sizeBytes: size ?? null,
        syncState: 'idle',
        verified_at: new Date().toISOString(),
      },
    ];

    return NextResponse.json(
      {
        storage_edges,
        status: memoryExists && blobExists ? 'verified' : 'failed',
        updatedAt: new Date().toISOString(),
      },
      { status: 200 }
    );
  } catch {
    return NextResponse.json({ error: 'INTERNAL_ERROR', message: 'Failed to verify upload' }, { status: 500 });
  }
}

// Mock database verification
async function verifyMemoryInDatabase(appMemoryId: string): Promise<boolean> {
  // TODO: Query Neon database to verify memory exists
  console.log(`🔍 Verifying memory ${appMemoryId} in Neon database...`);
  return true; // Mock: always return true
}

// Mock blob storage verification
async function verifyBlobInStorage(remoteId: string): Promise<boolean> {
  // TODO: Verify blob exists in S3/Vercel Blob/Arweave/IPFS
  console.log(`🔍 Verifying blob ${remoteId} in storage...`);
  return true; // Mock: always return true
}
