# AI Chatbot Schema Analysis - Corrected Import Guide

## 🚨 **Issue with Original Import Guide**

The original import guide in `docs/ai-chatbot-import-guide.md` contains **incorrect information** about the AI suggestions table. After analyzing the actual ai-chatbot repository, here's what we found:

## 📋 **Actual AI Chatbot Schema (What We Need)**

### **Core Tables Required:**

#### 1. **Chat Table** ✅

```typescript
export const chat = pgTable("Chat", {
  id: uuid("id").primaryKey().notNull().defaultRandom(),
  createdAt: timestamp("createdAt").notNull(),
  title: text("title").notNull(),
  userId: uuid("userId")
    .notNull()
    .references(() => user.id),
  visibility: varchar("visibility", { enum: ["public", "private"] })
    .notNull()
    .default("private"),
  lastContext: jsonb("lastContext").$type<LanguageModelV2Usage | null>(),
});
```

#### 2. **Message Table** ✅

```typescript
export const message = pgTable("Message_v2", {
  id: uuid("id").primaryKey().notNull().defaultRandom(),
  chatId: uuid("chatId")
    .notNull()
    .references(() => chat.id),
  role: varchar("role").notNull(),
  parts: json("parts").notNull(),
  attachments: json("attachments").notNull(),
  createdAt: timestamp("createdAt").notNull(),
});
```

#### 3. **Vote Table** ✅

```typescript
export const vote = pgTable(
  "Vote_v2",
  {
    chatId: uuid("chatId")
      .notNull()
      .references(() => chat.id),
    messageId: uuid("messageId")
      .notNull()
      .references(() => message.id),
    isUpvoted: boolean("isUpvoted").notNull(),
  },
  (table) => ({
    pk: primaryKey({ columns: [table.chatId, table.messageId] }),
  })
);
```

#### 4. **Document Table** ✅

```typescript
export const document = pgTable(
  "Document",
  {
    id: uuid("id").notNull().defaultRandom(),
    createdAt: timestamp("createdAt").notNull(),
    title: text("title").notNull(),
    content: text("content"),
    kind: varchar("text", { enum: ["text", "code", "image", "sheet"] })
      .notNull()
      .default("text"),
    userId: uuid("userId")
      .notNull()
      .references(() => user.id),
  },
  (table) => ({
    pk: primaryKey({ columns: [table.id, table.createdAt] }),
  })
);
```

#### 5. **Suggestion Table** ⚠️ **SPECIFIC TO DOCUMENTS**

```typescript
export const suggestion = pgTable(
  "Suggestion",
  {
    id: uuid("id").notNull().defaultRandom(),
    documentId: uuid("documentId").notNull(),
    documentCreatedAt: timestamp("documentCreatedAt").notNull(),
    originalText: text("originalText").notNull(),
    suggestedText: text("suggestedText").notNull(),
    description: text("description"),
    isResolved: boolean("isResolved").notNull().default(false),
    userId: uuid("userId")
      .notNull()
      .references(() => user.id),
    createdAt: timestamp("createdAt").notNull(),
  },
  (table) => ({
    pk: primaryKey({ columns: [table.id] }),
    documentRef: foreignKey({
      columns: [table.documentId, table.documentCreatedAt],
      foreignColumns: [document.id, document.createdAt],
    }),
  })
);
```

#### 6. **Stream Table** ✅

```typescript
export const stream = pgTable(
  "Stream",
  {
    id: uuid("id").notNull().defaultRandom(),
    chatId: uuid("chatId").notNull(),
    createdAt: timestamp("createdAt").notNull(),
  },
  (table) => ({
    pk: primaryKey({ columns: [table.id] }),
    chatRef: foreignKey({
      columns: [table.chatId],
      foreignColumns: [chat.id],
    }),
  })
);
```

## 🔍 **What the Suggestion Table Actually Does**

### **Purpose:**

- **Document Editing Suggestions**: AI-powered writing suggestions for documents
- **NOT for memories**: The suggestions are specifically for the `Document` table
- **Writing Assistant**: Helps improve text content in documents

### **How It Works:**

1. User creates a document
2. AI analyzes the document content
3. AI generates writing improvement suggestions
4. Suggestions are stored in the `suggestion` table
5. User can accept/reject suggestions

### **Key Insight:**

The `suggestion` table is **NOT** for memory curation or AI-Self functionality. It's specifically for **document editing assistance**.

## 🎯 **What We Actually Need for Futura**

### **For Basic AI Chatbot:**

```typescript
// Only these tables are needed for basic chat functionality
export const chats = pgTable("Chat", {
  /* ... */
});
export const messages = pgTable("Message_v2", {
  /* ... */
});
export const votes = pgTable("Vote_v2", {
  /* ... */
});
export const streams = pgTable("Stream", {
  /* ... */
});
```

### **For Document Editing (Optional):**

```typescript
// Only if you want AI-powered document editing
export const documents = pgTable("Document", {
  /* ... */
});
export const suggestions = pgTable("Suggestion", {
  /* ... */
});
```

### **For Memory AI Features (Custom):**

```typescript
// Custom table for memory-specific AI features
export const memoryAISuggestions = pgTable("MemoryAISuggestion", {
  id: uuid("id").primaryKey().notNull().defaultRandom(),
  memoryId: text("memoryId").notNull(),
  memoryType: text("memoryType", { enum: MEMORY_TYPES }).notNull(),
  suggestionType: text("suggestionType", {
    enum: ["tag", "description", "title", "legacy_message"],
  }).notNull(),
  originalContent: text("originalContent"),
  suggestedContent: text("suggestedContent").notNull(),
  description: text("description"),
  isResolved: boolean("isResolved").notNull().default(false),
  userId: text("userId")
    .notNull()
    .references(() => allUsers.id, { onDelete: "cascade" }),
  createdAt: timestamp("createdAt").notNull(),
});
```

## 🚨 **Corrections to Import Guide**

### **What Was Wrong:**

1. **Incorrect suggestion table**: The guide suggested a generic "AI suggestions for memories" table
2. **Missing context**: Didn't explain that suggestions are for document editing
3. **Wrong purpose**: Suggested it was for memory curation when it's for writing assistance

### **What's Actually Needed:**

1. **Core chat tables**: `chats`, `messages`, `votes`, `streams`
2. **Optional document tables**: Only if you want AI document editing
3. **Custom memory AI tables**: For memory-specific AI features (not in original ai-chatbot)

## 🎯 **Recommendation for Futura**

### **Phase 1: Basic AI Chat**

- Add: `chats`, `messages`, `votes`, `streams` tables
- Skip: `documents`, `suggestions` tables (not needed for basic chat)

### **Phase 2: Memory AI Features (Custom)**

- Create custom `memoryAISuggestions` table for memory-specific AI features
- This is NOT in the original ai-chatbot - it's specific to your use case

### **Phase 3: Document AI (Optional)**

- Add `documents` and `suggestions` tables if you want AI-powered document editing

## 📝 **Updated Schema for Futura**

```typescript
// Core AI Chat Tables (Required)
export const chats = pgTable("Chat", {
  id: uuid("id").primaryKey().notNull().defaultRandom(),
  createdAt: timestamp("createdAt").notNull(),
  title: text("title").notNull(),
  userId: text("userId")
    .notNull()
    .references(() => allUsers.id, { onDelete: "cascade" }),
  visibility: text("visibility", { enum: ["public", "private"] })
    .notNull()
    .default("private"),
  lastContext: jsonb("lastContext").$type<LanguageModelV2Usage | null>(),
});

export const messages = pgTable("Message_v2", {
  id: uuid("id").primaryKey().notNull().defaultRandom(),
  chatId: uuid("chatId")
    .notNull()
    .references(() => chats.id, { onDelete: "cascade" }),
  role: text("role").notNull(),
  parts: json("parts").notNull(),
  attachments: json("attachments").notNull(),
  createdAt: timestamp("createdAt").notNull(),
});

export const votes = pgTable(
  "Vote_v2",
  {
    chatId: uuid("chatId")
      .notNull()
      .references(() => chats.id),
    messageId: uuid("messageId")
      .notNull()
      .references(() => messages.id),
    isUpvoted: boolean("isUpvoted").notNull(),
  },
  (table) => ({
    pk: primaryKey({ columns: [table.chatId, table.messageId] }),
  })
);

export const streams = pgTable(
  "Stream",
  {
    id: uuid("id").primaryKey().notNull().defaultRandom(),
    chatId: uuid("chatId").notNull(),
    createdAt: timestamp("createdAt").notNull(),
  },
  (table) => ({
    pk: primaryKey({ columns: [table.id] }),
    chatRef: foreignKey({
      columns: [table.chatId],
      foreignColumns: [chats.id],
    }),
  })
);

// Custom Memory AI Table (For Futura-specific features)
export const memoryAISuggestions = pgTable("MemoryAISuggestion", {
  id: uuid("id").primaryKey().notNull().defaultRandom(),
  memoryId: text("memoryId").notNull(),
  memoryType: text("memoryType", { enum: MEMORY_TYPES }).notNull(),
  suggestionType: text("suggestionType", {
    enum: ["tag", "description", "title", "legacy_message"],
  }).notNull(),
  originalContent: text("originalContent"),
  suggestedContent: text("suggestedContent").notNull(),
  description: text("description"),
  isResolved: boolean("isResolved").notNull().default(false),
  userId: text("userId")
    .notNull()
    .references(() => allUsers.id, { onDelete: "cascade" }),
  createdAt: timestamp("createdAt").notNull(),
});
```

## 🎯 **Conclusion**

The original import guide was **misleading** about the suggestion table. The ai-chatbot's suggestion table is specifically for document editing, not for memory curation. For Futura's AI-Self features, we need to create custom tables for memory-specific AI suggestions.

**Next Steps:**

1. Add the core chat tables (`chats`, `messages`, `votes`, `streams`)
2. Create custom `memoryAISuggestions` table for memory-specific AI features
3. Skip the document-related tables unless you want AI document editing
