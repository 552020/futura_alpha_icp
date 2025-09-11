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

see the completed file

#### 2.3 Utility Functions Added ✅

see the completed file

### Phase 3: AI Configuration

#### 3.1 AI Models Configuration

```typescript
// Create src/nextjs/src/lib/ai/models.ts
export const DEFAULT_CHAT_MODEL: string = "chat-model";

export interface ChatModel {
  id: string;
  name: string;
  description: string;
}

export const chatModels: Array<ChatModel> = [
  {
    id: "chat-model",
    name: "Grok Vision",
    description: "Advanced multimodal model with vision and text capabilities",
  },
  {
    id: "chat-model-reasoning",
    name: "Grok Reasoning",
    description: "Uses advanced chain-of-thought reasoning for complex problems",
  },
];
```

#### 3.2 AI Providers Setup

```typescript
// Create src/nextjs/src/lib/ai/providers.ts
import { createOpenAI } from "@ai-sdk/openai";
import { createGateway } from "@ai-sdk/gateway";

export const gateway = createGateway({
  apiKey: process.env.AI_GATEWAY_API_KEY,
});

export const openai = createOpenAI({
  apiKey: process.env.OPENAI_API_KEY,
});
```

### Phase 4: Core Components Import

#### 4.1 Chat Components to Import

```typescript
// Import from ai-chatbot/components/
-chat.tsx - // Main chat interface
  message.tsx - // Message rendering
  messages.tsx - // Messages container
  multimodal -
  input.tsx - // Input with file support
  message -
  actions.tsx - // Message actions (edit, delete, etc.)
  suggested -
  actions.tsx; // AI-powered suggestions
```

#### 4.2 AI-Specific Components

```typescript
// Import from ai-chatbot/components/elements/
-message.tsx - // Enhanced message rendering
  response.tsx - // AI response handling
  reasoning.tsx - // Chain-of-thought reasoning
  tool.tsx - // AI tool integration
  suggestion.tsx; // AI suggestions display
```

#### 4.3 UI Components to Import

```typescript
// Import from ai-chatbot/components/ui/
-scroll -
  area.tsx - // Chat scrolling
  textarea.tsx - // Enhanced text input
  tooltip.tsx - // Tooltips for AI features
  progress.tsx; // Loading states
```

### Phase 5: API Routes Integration

#### 5.1 Chat API Routes

```typescript
// Create src/nextjs/src/app/api/chat/
-route.ts - // Main chat endpoint
  [id] / stream / route.ts - // Streaming responses
  schema.ts - // Chat request/response schemas
  vote / route.ts - // Message voting
  suggestions / route.ts; // AI suggestions
```

#### 5.2 Memory AI Integration

```typescript
// Extend existing memory routes
-/api/eeimmors / [id] / analyze - // AI analysis of memories
  /api/eeimmors / [id] / suggest - // AI suggestions for memory
  /api/eeimmors / curate; // AI-powered memory curation
```

### Phase 6: AI Tools for Memory Management

#### 6.1 Memory-Specific AI Tools

```typescript
// Create src/nextjs/src/lib/ai/tools/
-analyze -
  memory.ts - // Analyze memory content
  suggest -
  tags.ts - // Suggest memory tags
  generate -
  description.ts - // Generate memory descriptions
  curate -
  gallery.ts - // AI gallery curation
  legacy -
  message.ts; // Generate legacy messages
```

#### 6.2 Integration with Existing Memory System

```typescript
// Extend existing memory services
export class MemoryAIService {
  async analyzeMemory(memoryId: string): Promise<MemoryAnalysis> {
    // AI analysis of memory content
  }

  async suggestTags(memoryId: string): Promise<string[]> {
    // AI-powered tag suggestions
  }

  async generateLegacyMessage(memoryId: string): Promise<string> {
    // Generate legacy messages for AI-Self
  }
}
```

## Integration Points

### 1. Authentication Integration

- **Current**: Internet Identity + NextAuth
- **AI Chatbot**: Standard NextAuth
- **Solution**: Extend existing auth to support AI chat sessions

### 2. Database Integration

- **Current**: Users, Memories, Galleries tables
- **AI Chatbot**: Users, Chats, Messages tables
- **Solution**: Add chat tables with foreign keys to existing users

### 3. Storage Integration

- **Current**: Vercel Blob + ICP Canister
- **AI Chatbot**: Vercel Blob only
- **Solution**: Use existing storage for AI-generated content

### 4. UI Integration

- **Current**: shadcn/ui components
- **AI Chatbot**: shadcn/ui components
- **Solution**: Import compatible components, adapt styling

## Implementation Steps

### Step 1: Setup AI Infrastructure

1. Install AI SDK dependencies
2. Configure environment variables
3. Set up AI providers and models

### Step 2: Database Migration

1. Create chat-related tables
2. Add AI suggestion tables
3. Update existing schemas

### Step 3: Core Components

1. Import and adapt chat components
2. Integrate with existing UI system
3. Add AI-specific features

### Step 4: API Integration

1. Create chat API endpoints
2. Integrate with existing memory APIs
3. Add AI-powered memory features

### Step 5: AI Tools Development

1. Create memory-specific AI tools
2. Integrate with existing memory system
3. Add AI-Self functionality

### Step 6: Testing & Optimization

1. Test AI features with existing data
2. Optimize performance and costs
3. Add monitoring and analytics

## File Structure After Integration

```
src/nextjs/src/
├── app/
│   ├── api/
│   │   ├── chat/              # New AI chat endpoints
│   │   └── memories/
│   │       └── [id]/
│   │           ├── analyze/   # AI memory analysis
│   │           └── suggest/   # AI suggestions
│   └── [lang]/
│       └── ai-chat/           # New AI chat pages
├── components/
│   ├── ai/                    # New AI components
│   │   ├── chat.tsx
│   │   ├── message.tsx
│   │   └── suggestions.tsx
│   └── memory/
│       └── ai-enhanced/       # AI-enhanced memory components
├── lib/
│   ├── ai/                    # New AI configuration
│   │   ├── models.ts
│   │   ├── providers.ts
│   │   └── tools/
│   └── db/
│       └── schema.ts          # Extended with chat tables
```

## Benefits of Integration

### For AI-Self (Aftermath)

- **Legacy Message Generation**: AI-powered creation of meaningful legacy messages
- **Memory Curation**: Intelligent organization and tagging of memories
- **Conversational Interface**: Natural language interaction with stored memories

### For Memory Management

- **Smart Tagging**: AI-powered tag suggestions
- **Content Analysis**: Automatic description generation
- **Gallery Curation**: AI-assisted gallery organization

### For User Experience

- **Intelligent Search**: Natural language memory search
- **Personalized Suggestions**: AI-powered memory recommendations
- **Interactive Storytelling**: Chat-based memory exploration

## Considerations

### Performance

- **Streaming Responses**: Implement efficient streaming for real-time AI responses
- **Caching**: Cache AI responses to reduce API costs
- **Rate Limiting**: Implement rate limiting for AI features

### Costs

- **AI API Costs**: Monitor and optimize AI model usage
- **Storage Costs**: Efficient storage of AI-generated content
- **Compute Costs**: Optimize AI processing for cost efficiency

### Privacy

- **Data Privacy**: Ensure AI processing respects user privacy
- **Content Filtering**: Implement appropriate content filtering
- **User Control**: Give users control over AI features

## Conclusion

This integration will transform your Futura application into an AI-powered memory platform, enabling the AI-Self (Aftermath) vertical and enhancing memory management capabilities. The modular approach allows for gradual implementation while maintaining compatibility with your existing Internet Identity authentication and ICP storage systems.

The key is to start with core AI infrastructure and gradually add memory-specific AI features, ensuring seamless integration with your existing user experience and data architecture.
