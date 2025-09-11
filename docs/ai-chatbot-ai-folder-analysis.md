# AI Chatbot AI Folder Analysis

## Overview

This document provides a file-by-file analysis of the `secretus/ai-chatbot/lib/ai/` folder to determine what needs to be copied and adapted for integration into the Futura Next.js application.

## Folder Structure

```
lib/ai/
├── models.ts              # Model definitions and configurations
├── providers.ts           # AI provider setup and configuration
├── prompts.ts             # Prompt templates and system prompts
├── entitlements.ts        # User entitlements and rate limiting
├── models.mock.ts         # Mock models for testing
├── models.test.ts         # Unit tests for models
└── tools/                 # AI tools directory
    ├── create-document.ts # Document creation tool
    ├── update-document.ts # Document update tool
    ├── request-suggestions.ts # AI suggestions tool
    └── get-weather.ts     # Weather information tool
```

## File-by-File Analysis

### 1. `models.ts` - Model Definitions ✅ COPY AS-IS

**Purpose**: Defines available AI models and their configurations.

**Content**:

```typescript
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

**Integration Notes**:

- ✅ **Copy as-is** - No modifications needed
- ✅ **Dependencies**: None (pure TypeScript)
- ✅ **Compatibility**: Works with existing Futura setup

### 2. `providers.ts` - AI Provider Configuration ✅ COPY WITH MINOR ADAPTATIONS

**Purpose**: Sets up AI providers using Vercel AI Gateway and XAI models.

**Content**:

```typescript
import { customProvider, extractReasoningMiddleware, wrapLanguageModel } from "ai";
import { gateway } from "@ai-sdk/gateway";
import { isTestEnvironment } from "../constants";

export const myProvider = isTestEnvironment
  ? (() => {
      const { artifactModel, chatModel, reasoningModel, titleModel } = require("./models.mock");
      return customProvider({
        languageModels: {
          "chat-model": chatModel,
          "chat-model-reasoning": reasoningModel,
          "title-model": titleModel,
          "artifact-model": artifactModel,
        },
      });
    })()
  : customProvider({
      languageModels: {
        "chat-model": gateway.languageModel("xai/grok-2-vision-1212"),
        "chat-model-reasoning": wrapLanguageModel({
          model: gateway.languageModel("xai/grok-3-mini"),
          middleware: extractReasoningMiddleware({ tagName: "think" }),
        }),
        "title-model": gateway.languageModel("xai/grok-2-1212"),
        "artifact-model": gateway.languageModel("xai/grok-2-1212"),
      },
    });
```

**Integration Notes**:

- ✅ **Copy with adaptations** - Need to create `constants.ts` file
- ⚠️ **Dependencies**: Requires `../constants` file
- ✅ **Environment Variables**: Uses existing `AI_GATEWAY_API_KEY`

### 3. `prompts.ts` - Prompt Templates ✅ COPY WITH ADAPTATIONS

**Purpose**: Contains all AI prompt templates and system prompts.

**Key Features**:

- **Artifacts Integration**: Special prompts for document creation
- **Geographic Context**: Location-based request hints
- **Model-Specific Prompts**: Different prompts for reasoning vs. regular models
- **Code Generation**: Python-focused code generation
- **Document Updates**: Context-aware document improvements

**Integration Notes**:

- ⚠️ **Copy with adaptations** - Need to adapt import paths
- ⚠️ **Dependencies**: References `@/components/artifact` and `@vercel/functions`
- ✅ **Core Logic**: Can be copied, just need to fix imports

### 4. `entitlements.ts` - User Entitlements ✅ COPY WITH ADAPTATIONS

**Purpose**: Defines user entitlements and rate limiting based on user type.

**Content**:

```typescript
import type { UserType } from "@/app/(auth)/auth";
import type { ChatModel } from "./models";

interface Entitlements {
  maxMessagesPerDay: number;
  availableChatModelIds: Array<ChatModel["id"]>;
}

export const entitlementsByUserType: Record<UserType, Entitlements> = {
  guest: {
    maxMessagesPerDay: 20,
    availableChatModelIds: ["chat-model", "chat-model-reasoning"],
  },
  regular: {
    maxMessagesPerDay: 100,
    availableChatModelIds: ["chat-model", "chat-model-reasoning"],
  },
};
```

**Integration Notes**:

- ⚠️ **Copy with adaptations** - Need to adapt `UserType` import
- ⚠️ **Dependencies**: References `@/app/(auth)/auth` (Futura uses different auth structure)
- ✅ **Core Logic**: Rate limiting logic can be adapted to Futura's user system

### 5. `models.mock.ts` - Mock Models for Testing ✅ COPY AS-IS

**Purpose**: Provides mock language models for testing environments.

**Content**:

```typescript
import type { LanguageModel } from "ai";

const createMockModel = (): LanguageModel => {
  return {
    specificationVersion: "v2",
    provider: "mock",
    modelId: "mock-model",
    // ... mock implementation
  } as unknown as LanguageModel;
};

export const chatModel = createMockModel();
export const reasoningModel = createMockModel();
export const titleModel = createMockModel();
export const artifactModel = createMockModel();
```

**Integration Notes**:

- ✅ **Copy as-is** - No modifications needed
- ✅ **Dependencies**: Only requires `ai` package (already added)
- ✅ **Testing**: Essential for development and testing

### 6. `models.test.ts` - Unit Tests ✅ COPY WITH ADAPTATIONS

**Purpose**: Unit tests for AI models using AI SDK testing utilities.

**Integration Notes**:

- ⚠️ **Copy with adaptations** - Need to adapt test utilities import
- ⚠️ **Dependencies**: References `@/tests/prompts/utils` (Futura may not have this)
- ✅ **Core Testing**: Can be adapted to Futura's testing setup

### 7. `tools/create-document.ts` - Document Creation Tool ⚠️ COPY WITH MAJOR ADAPTATIONS

**Purpose**: AI tool for creating documents with real-time streaming.

**Key Features**:

- **Real-time Streaming**: Uses `UIMessageStreamWriter` for live updates
- **Artifact Integration**: Creates documents in artifact system
- **Multiple Formats**: Supports text, code, and spreadsheet documents

**Integration Notes**:

- ⚠️ **Copy with major adaptations** - Heavy dependencies on artifact system
- ⚠️ **Dependencies**:
  - `@/lib/artifacts/server` (Futura doesn't have artifacts system)
  - `@/lib/types` (ChatMessage type)
  - `@/lib/db/queries` (Database queries)
- 🔄 **Adaptation Strategy**: May need to simplify or create Futura-specific version

### 8. `tools/update-document.ts` - Document Update Tool ⚠️ COPY WITH MAJOR ADAPTATIONS

**Purpose**: AI tool for updating existing documents.

**Integration Notes**:

- ⚠️ **Copy with major adaptations** - Same dependencies as create-document
- ⚠️ **Dependencies**: Same as create-document tool
- 🔄 **Adaptation Strategy**: May need to simplify for Futura's needs

### 9. `tools/request-suggestions.ts` - AI Suggestions Tool ⚠️ COPY WITH ADAPTATIONS

**Purpose**: AI tool for generating writing suggestions for documents.

**Key Features**:

- **Streaming Suggestions**: Real-time suggestion generation
- **Database Integration**: Saves suggestions to database
- **Writing Assistant**: AI-powered writing improvements

**Integration Notes**:

- ⚠️ **Copy with adaptations** - Database integration needed
- ⚠️ **Dependencies**:
  - `@/lib/db/queries` (getDocumentById, saveSuggestions)
  - `@/lib/db/schema` (Suggestion type)
- ✅ **Core Logic**: Can be adapted to work with Futura's database

### 10. `tools/get-weather.ts` - Weather Tool ✅ COPY AS-IS

**Purpose**: Simple weather information tool (optional).

**Integration Notes**:

- ✅ **Copy as-is** - Simple tool with no external dependencies
- ✅ **Optional**: Can be implemented later or skipped entirely

## Required Dependencies

### Files to Create in Futura

1. **`src/lib/constants.ts`** - Environment detection constants
2. **`src/lib/ai/`** - Complete AI folder structure
3. **Database queries** - For document and suggestion management
4. **Type definitions** - For ChatMessage and other AI types

### Dependencies Already Available

- ✅ `@ai-sdk/gateway` - Already added
- ✅ `@ai-sdk/provider` - Already added
- ✅ `@ai-sdk/react` - Already added
- ✅ `@ai-sdk/xai` - Already added
- ✅ `ai` - Already added
- ✅ `zod` - Already available in Futura

### Dependencies to Add

- ⚠️ `@vercel/functions` - For Geo types (optional)
- ⚠️ `ai/test` - For testing utilities (optional)

## Copy Strategy

### Phase 1: Core AI Configuration (Copy As-Is)

1. ✅ `models.ts` - Copy directly
2. ✅ `models.mock.ts` - Copy directly
3. ⚠️ `providers.ts` - Copy with constants.ts creation
4. ⚠️ `prompts.ts` - Copy with import path fixes

### Phase 2: User Management (Copy with Adaptations)

1. ⚠️ `entitlements.ts` - Adapt to Futura's user system
2. ⚠️ `constants.ts` - Create new file

### Phase 3: AI Tools (Copy with Major Adaptations)

1. ⚠️ `tools/get-weather.ts` - Copy as-is (optional)
2. 🔄 `tools/request-suggestions.ts` - Adapt database integration
3. 🔄 `tools/create-document.ts` - Simplify or create Futura version
4. 🔄 `tools/update-document.ts` - Simplify or create Futura version

### Phase 4: Testing (Copy with Adaptations)

1. ⚠️ `models.test.ts` - Adapt to Futura's testing setup

## Integration Priority

### High Priority (Essential for Basic Chat)

1. `models.ts` - Model definitions
2. `providers.ts` - AI provider setup
3. `prompts.ts` - Basic prompts
4. `constants.ts` - Environment detection

### Medium Priority (Enhanced Features)

1. `entitlements.ts` - User rate limiting
2. `models.mock.ts` - Testing support
3. `tools/request-suggestions.ts` - AI suggestions

### Low Priority (Advanced Features)

1. `tools/create-document.ts` - Document creation
2. `tools/update-document.ts` - Document updates
3. `tools/get-weather.ts` - Weather tool
4. `models.test.ts` - Unit tests

## Conclusion

**Yes, we should copy the entire `ai/` folder**, but with a phased approach:

1. **Start with core files** (models, providers, prompts) for basic AI chat functionality
2. **Adapt user management** (entitlements, constants) to Futura's system
3. **Simplify or skip complex tools** (document creation) initially
4. **Add testing support** once core functionality is working

The AI folder provides a complete, production-ready AI system that can be adapted to Futura's needs while maintaining the core functionality and architecture.


