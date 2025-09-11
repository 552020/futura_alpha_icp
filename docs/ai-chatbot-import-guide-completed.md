# AI Chatbot Import Guide for Futura Next.js App

## Overview

This guide outlines how to integrate AI chatbot functionalities from the `ai-chatbot` starter repository into your existing Futura Next.js application located in `src/nextjs`. The integration will enable AI-powered features for the **AI-Self (Aftermath)** vertical and enhance memory curation capabilities.

## Current State Analysis

### Existing Futura Next.js App Structure

- **Framework**: Next.js 15.5.0 with App Router
- **Database**: PostgreSQL with Drizzle ORM
- **Authentication**: NextAuth.js with Internet Identity integration
- **UI**: shadcn/ui components with Tailwind CSS
- **Storage**: Vercel Blob + ICP Canister storage
- **Current Chat**: Basic LiveChat integration (customer support)

### Key Differences from AI-Chatbot

- **No AI SDK**: Missing AI SDK and model providers
- **No Chat Schema**: No chat/message database tables
- **No Streaming**: No real-time AI response streaming
- **No AI Tools**: No AI-powered tools or suggestions
- **Different Auth**: Internet Identity vs. standard NextAuth

## Import Strategy

### Phase 1: Core AI Infrastructure

#### 1.1 Dependencies to Add

```json
{
  "dependencies": {
    "@ai-sdk/gateway": "^1.0.15",
    "@ai-sdk/provider": "2.0.0",
    "@ai-sdk/react": "2.0.26",
    "@ai-sdk/xai": "2.0.13",
    "ai": "5.0.26",
    "nanoid": "^5.0.8",
    "use-stick-to-bottom": "^1.1.1"
  }
}
```

#### 1.2 Environment Variables to Add

```env
# AI Gateway Configuration
AI_GATEWAY_API_KEY=your_gateway_key
NEXT_PUBLIC_AI_GATEWAY_URL=https://gateway.vercel.ai

# Model Configuration
NEXT_PUBLIC_DEFAULT_CHAT_MODEL=chat-model
NEXT_PUBLIC_REASONING_MODEL=chat-model-reasoning
```

### Phase 2: Database Schema Extensions ✅ COMPLETED

#### 2.1 Dependencies Added ✅

```json
{
  "dependencies": {
    "@ai-sdk/gateway": "^1.0.15",
    "@ai-sdk/provider": "2.0.0",
    "@ai-sdk/react": "2.0.26",
    "@ai-sdk/xai": "2.0.13",
    "ai": "5.0.26",
    "nanoid": "^5.0.8",
    "use-stick-to-bottom": "^1.1.1"
  }
}
```

#### 2.2 Database Schema - 1:1 Copy from AI-Chatbot ✅

**Tables Added to `src/nextjs/src/db/schema.ts`:**

```typescript
// Chat sessions
export const chat = pgTable("Chat", {
  id: uuid("id").primaryKey().notNull().defaultRandom(),
  createdAt: timestamp("createdAt").notNull(),
  title: text("title").notNull(),
  userId: uuid("userId")
    .notNull()
    .references(() => allUsers.id, { onDelete: "cascade" }),
  visibility: text("visibility", { enum: ["public", "private"] })
    .notNull()
    .default("private"),
  lastContext: jsonb("lastContext").$type<LanguageModelV2Usage | null>(),
});

// Messages (v2 with parts structure)
export const message = pgTable("Message_v2", {
  id: uuid("id").primaryKey().notNull().defaultRandom(),
  chatId: uuid("chatId")
    .notNull()
    .references(() => chat.id, { onDelete: "cascade" }),
  role: text("role").notNull(),
  parts: json("parts").notNull(),
  attachments: json("attachments").notNull(),
  createdAt: timestamp("createdAt").notNull(),
});

// Legacy messages (for backward compatibility)
export const messageDeprecated = pgTable("Message", {
  id: uuid("id").primaryKey().notNull().defaultRandom(),
  chatId: uuid("chatId")
    .notNull()
    .references(() => chat.id, { onDelete: "cascade" }),
  role: text("role").notNull(),
  content: text("content").notNull(),
  createdAt: timestamp("createdAt").notNull(),
});

// Voting system for messages
export const vote = pgTable("Vote_v2", {
  id: uuid("id").primaryKey().notNull().defaultRandom(),
  messageId: uuid("messageId")
    .notNull()
    .references(() => message.id, { onDelete: "cascade" }),
  userId: uuid("userId")
    .notNull()
    .references(() => allUsers.id, { onDelete: "cascade" }),
  isUpvoted: boolean("isUpvoted").notNull(),
  createdAt: timestamp("createdAt").notNull(),
});

// Legacy voting (for backward compatibility)
export const voteDeprecated = pgTable("Vote", {
  id: uuid("id").primaryKey().notNull().defaultRandom(),
  messageId: uuid("messageId")
    .notNull()
    .references(() => messageDeprecated.id, { onDelete: "cascade" }),
  userId: uuid("userId")
    .notNull()
    .references(() => allUsers.id, { onDelete: "cascade" }),
  isUpvoted: boolean("isUpvoted").notNull(),
  createdAt: timestamp("createdAt").notNull(),
});

// Document management for AI processing
export const document = pgTable("Document", {
  id: uuid("id").primaryKey().notNull().defaultRandom(),
  createdAt: timestamp("createdAt").notNull(),
  name: text("name").notNull(),
  url: text("url").notNull(),
  userId: uuid("userId")
    .notNull()
    .references(() => allUsers.id, { onDelete: "cascade" }),
});

// AI suggestions for document editing (not memory curation)
export const suggestion = pgTable("Suggestion", {
  id: uuid("id").primaryKey().notNull().defaultRandom(),
  documentId: uuid("documentId").notNull(),
  documentCreatedAt: timestamp("documentCreatedAt").notNull(),
  originalText: text("originalText").notNull(),
  suggestedText: text("suggestedText").notNull(),
  description: text("description"),
  isResolved: boolean("isResolved").notNull().default(false),
  userId: uuid("userId")
    .notNull()
    .references(() => allUsers.id, { onDelete: "cascade" }),
  createdAt: timestamp("createdAt").notNull(),
});

// Streaming session management
export const stream = pgTable("Stream", {
  id: uuid("id").primaryKey().notNull().defaultRandom(),
  createdAt: timestamp("createdAt").notNull(),
  userId: uuid("userId")
    .notNull()
    .references(() => allUsers.id, { onDelete: "cascade" }),
});
```

#### 2.3 Utility Functions Added ✅

**Added to `src/nextjs/src/lib/utils.ts`:**

```typescript
import { generateId } from "ai";

/**
 * Generate a unique UUID for database records
 * Uses the AI SDK's generateId function for consistency
 */
export function generateUUID(): string {
  return generateId();
}
```

#### 2.4 Migration Executed ✅

- **Migration File**: `src/db/migrations/0025_regular_bloodaxe.sql`
- **Command Used**: `pnpm db:push` (to avoid conflicts with existing tables)
- **Result**: All AI chatbot tables successfully created with proper foreign key relationships to `allUsers.id`

#### 2.5 Key Adaptations Made

1. **User Table Integration**: All `userId` references point to `allUsers.id` instead of a separate `users` table
2. **Cascade Deletes**: Added `{ onDelete: "cascade" }` to maintain data integrity
3. **1:1 Schema Copy**: Preserved exact structure from ai-chatbot for full compatibility
4. **TypeScript Types**: Exported corresponding TypeScript types for all new tables
