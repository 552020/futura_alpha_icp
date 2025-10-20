# Card Upload/Processing Status Indicator

## Status

- State: Open
- Priority: Medium
- Area: Dashboard UI, Upload Flow

## Problem

When a memory is uploading or being processed (derivatives, storage edges), the dashboard cards do not clearly communicate per-item status. Users may click or navigate without knowing whether an item is still in progress.

## Goal

Show per-item status on the dashboard card footer and/or thumbnail area, so users see:

- Uploading (progress)
- Processing (derivatives generation)
- Finalizing (API / storage-edge writes)
- Completed
- Error (retry option)

## UX Requirements

- Replace action buttons with a compact spinner during operations affecting that item (already done for deletion; extend to upload/processing).
- Optional progress bar (thin, top or bottom of the card) for upload progress when available.
- Subtle overlay badge on the thumbnail: “Uploading…”, “Processing…”, “Finalizing…”, “Error”.
- Click disabled while in non-interactive states, with tooltip text.

## States to Represent

1. Uploading original file (percent if possible)
2. Processing derivatives (display/thumb/placeholder, etc.)
3. Finalizing (write records, create storage edges)
4. Completed (normal card)
5. Error (show warning badge and retry)

## Data/Events Sources

- Client upload hooks: `src/nextjs/src/services/upload/*` (s3-with-processing.ts, single-file-processor.ts, finalize.ts)
- API finalize endpoint: `/api/upload/complete`
- Memory creation flow: `createMemoryWithAssets` (service)
- Storage edges: `createMemoryStorageEdges`

We should emit a client-side status stream per item id (temporary client id -> server memory id), e.g. via:

- React Query mutation state + custom event emitter, or
- A lightweight upload context that tracks itemId -> status

## Technical Plan

- Add a lightweight status store/hook:
  - `src/nextjs/src/hooks/use-upload-status.ts`
  - API: `setStatus(tempId|memoryId, status, progress?)`, `getStatus(id)`, `subscribe(id, cb)`
- When starting upload (in `item-upload-button.tsx` / `use-file-upload` path), register a temp client id for the card (before id exists) and update progress.
- When memory id is returned, map temp id -> memory id and continue updates.
- In `MemoryGrid`/`ContentCard`:
  - Read status for each `memory.id`; if missing, attempt temp-id linkage when available.
  - Pass `isUploading`, `isProcessing`, `progress`, `hasError` down to `BaseCard`.
- `BaseCard`:
  - Show spinner/progress bar and disable actions based on state.
  - Small badge overlay on thumbnail.

## Components/Files to Touch

- UI
  - `src/nextjs/src/components/common/base-card.tsx`
  - `src/nextjs/src/components/common/content-card.tsx`
  - `src/nextjs/src/components/memory/memory-grid.tsx`
- Hooks/State
  - `src/nextjs/src/hooks/use-upload-status.ts` (new)
- Upload Flow
  - `src/nextjs/src/components/memory/item-upload-button.tsx`
  - `src/nextjs/src/services/upload/single-file-processor.ts`
  - `src/nextjs/src/services/upload/s3-with-processing.ts`
  - `src/nextjs/src/services/upload/finalize.ts`

## React Query

- Keep dashboard query invalidation on finalize to refresh final card data.
- Avoid global pending flags; use per-item status from the new hook.

## Acceptance Criteria

- Card shows an uploading spinner/progress for in-flight uploads only for that specific item.
- Card shows “Processing” during derivative generation/finalize.
- Card returns to normal state on completion; on errors, shows error badge and a retry action.
- No UI flicker across other cards; only targeted item changes state.

## Follow-ups

- Extend to folder operations if needed (bulk progress display).
- Telemetry for time spent per stage (upload, processing, finalize).
