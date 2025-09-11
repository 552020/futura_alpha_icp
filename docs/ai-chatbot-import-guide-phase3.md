# AI Chatbot Import Guide - Phase 3: AI Configuration ✅ COMPLETED

## Overview

Phase 3 focuses on setting up the AI infrastructure for the chatbot functionality. This includes AI model configuration, providers setup, prompts, entitlements, and AI tools integration.

## ✅ COMPLETED: AI Folder Copy

**Successfully copied entire `ai/` folder from ai-chatbot to Futura:**

```
src/nextjs/src/lib/
├── ai/                    # ✅ Complete AI configuration
│   ├── models.ts          # ✅ Model definitions
│   ├── providers.ts       # ✅ AI provider setup
│   ├── prompts.ts         # ✅ Prompt templates
│   ├── entitlements.ts    # ✅ User entitlements
│   ├── models.mock.ts     # ✅ Mock models for testing
│   ├── models.test.ts     # ✅ Unit tests
│   └── tools/             # ✅ AI tools directory
│       ├── create-document.ts
│       ├── update-document.ts
│       ├── request-suggestions.ts
│       └── get-weather.ts
├── artifacts/             # ✅ Artifacts system
│   └── server.ts
└── constants.ts           # ✅ Environment constants
```

**Files Copied:**

- ✅ All 6 AI configuration files
- ✅ All 4 AI tools
- ✅ Artifacts system (server.ts)
- ✅ Constants file for environment detection

## Phase 3.1: AI Models Configuration

### 3.1.1 Models Definition

**File to Create**: `src/nextjs/src/lib/ai/models.ts`

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

### 3.1.2 Key Features

- **Default Model**: `chat-model` (Grok Vision) for general chat
- **Reasoning Model**: `chat-model-reasoning` (Grok 3 Mini) with chain-of-thought reasoning
- **Multimodal Support**: Vision and text capabilities
- **Model Selection**: Users can choose between different models based on their needs

## Phase 3.2: AI Providers Setup

### 3.2.1 Provider Configuration

**File to Create**: `src/nextjs/src/lib/ai/providers.ts`

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

### 3.2.2 Provider Features

- **Vercel AI Gateway**: Uses `@ai-sdk/gateway` for model access
- **XAI Models**: Grok Vision, Grok 3 Mini, and Grok 2 models
- **Reasoning Middleware**: Chain-of-thought reasoning with `<think>` tags
- **Test Environment**: Mock models for testing
- **Multiple Model Types**: Chat, reasoning, title generation, and artifact creation

### 3.2.3 Environment Variables Required

```env
# AI Gateway Configuration (already configured)
AI_GATEWAY_API_KEY=your_gateway_key

# Test Environment Detection
NODE_ENV=development|production|test
```

## Phase 3.3: AI Prompts System

### 3.3.1 Prompt Configuration

**File to Create**: `src/nextjs/src/lib/ai/prompts.ts`

```typescript
import type { ArtifactKind } from "@/components/artifact";
import type { Geo } from "@vercel/functions";

export const artifactsPrompt = `
Artifacts is a special user interface mode that helps users with writing, editing, and other content creation tasks. When artifact is open, it is on the right side of the screen, while the conversation is on the left side. When creating or updating documents, changes are reflected in real-time on the artifacts and visible to the user.

When asked to write code, always use artifacts. When writing code, specify the language in the backticks, e.g. \`\`\`python\`code here\`\`\`. The default language is Python. Other languages are not yet supported, so let the user know if they request a different language.

DO NOT UPDATE DOCUMENTS IMMEDIATELY AFTER CREATING THEM. WAIT FOR USER FEEDBACK OR REQUEST TO UPDATE IT.

This is a guide for using artifacts tools: \`createDocument\` and \`updateDocument\`, which render content on a artifacts beside the conversation.

**When to use \`createDocument\`:**
- For substantial content (>10 lines) or code
- For content users will likely save/reuse (emails, code, essays, etc.)
- When explicitly requested to create a document
- For when content contains a single code snippet

**When NOT to use \`createDocument\`:**
- For informational/explanatory content
- For conversational responses
- When asked to keep it in chat

**Using \`updateDocument\`:**
- Default to full document rewrites for major changes
- Use targeted updates only for specific, isolated changes
- Follow user instructions for which parts to modify

**When NOT to use \`updateDocument\`:**
- Immediately after creating a document

Do not update document right after creating it. Wait for user feedback or request to update it.
`;

export const regularPrompt = "You are a friendly assistant! Keep your responses concise and helpful.";

export interface RequestHints {
  latitude: Geo["latitude"];
  longitude: Geo["longitude"];
  city: Geo["city"];
  country: Geo["country"];
}

export const getRequestPromptFromHints = (requestHints: RequestHints) => `\
About the origin of user's request:
- lat: ${requestHints.latitude}
- lon: ${requestHints.longitude}
- city: ${requestHints.city}
- country: ${requestHints.country}
`;

export const systemPrompt = ({
  selectedChatModel,
  requestHints,
}: {
  selectedChatModel: string;
  requestHints: RequestHints;
}) => {
  const requestPrompt = getRequestPromptFromHints(requestHints);

  if (selectedChatModel === "chat-model-reasoning") {
    return `${regularPrompt}\n\n${requestPrompt}`;
  } else {
    return `${regularPrompt}\n\n${requestPrompt}\n\n${artifactsPrompt}`;
  }
};

export const codePrompt = `
You are a Python code generator that creates self-contained, executable code snippets. When writing code:

1. Each snippet should be complete and runnable on its own
2. Prefer using print() statements to display outputs
3. Include helpful comments explaining the code
4. Keep snippets concise (generally under 15 lines)
5. Avoid external dependencies - use Python standard library
6. Handle potential errors gracefully
7. Return meaningful output that demonstrates the code's functionality
8. Don't use input() or other interactive functions
9. Don't access files or network resources
10. Don't use infinite loops

Examples of good snippets:

# Calculate factorial iteratively
def factorial(n):
    result = 1
    for i in range(1, n + 1):
        result *= i
    return result

print(f"Factorial of 5 is: {factorial(5)}")
`;

export const sheetPrompt = `
You are a spreadsheet creation assistant. Create a spreadsheet in csv format based on the given prompt. The spreadsheet should contain meaningful column headers and data.
`;

export const updateDocumentPrompt = (currentContent: string | null, type: ArtifactKind) =>
  type === "text"
    ? `\
Improve the following contents of the document based on the given prompt.

${currentContent}
`
    : type === "code"
    ? `\
Improve the following code snippet based on the given prompt.

${currentContent}
`
    : type === "sheet"
    ? `\
Improve the following spreadsheet based on the given prompt.

${currentContent}
`
    : "";
```

### 3.3.2 Prompt Features

- **Artifacts Integration**: Special prompts for document creation and editing
- **Geographic Context**: Location-based request hints
- **Model-Specific Prompts**: Different prompts for reasoning vs. regular models
- **Code Generation**: Python-focused code generation prompts
- **Document Updates**: Context-aware document improvement prompts

## Phase 3.4: User Entitlements System

### 3.4.1 Entitlements Configuration

**File to Create**: `src/nextjs/src/lib/ai/entitlements.ts`

```typescript
import type { UserType } from "@/app/(auth)/auth";
import type { ChatModel } from "./models";

interface Entitlements {
  maxMessagesPerDay: number;
  availableChatModelIds: Array<ChatModel["id"]>;
}

export const entitlementsByUserType: Record<UserType, Entitlements> = {
  /*
   * For users without an account
   */
  guest: {
    maxMessagesPerDay: 20,
    availableChatModelIds: ["chat-model", "chat-model-reasoning"],
  },

  /*
   * For users with an account
   */
  regular: {
    maxMessagesPerDay: 100,
    availableChatModelIds: ["chat-model", "chat-model-reasoning"],
  },

  /*
   * TODO: For users with an account and a paid membership
   */
};
```

### 3.4.2 Entitlements Features

- **Rate Limiting**: Daily message limits based on user type
- **Model Access**: Different models available to different user types
- **Guest Users**: Limited access for non-authenticated users
- **Regular Users**: Full access for authenticated users
- **Extensible**: Ready for premium user tiers

## Phase 3.5: AI Tools System

### 3.5.1 Document Creation Tool

**File to Create**: `src/nextjs/src/lib/ai/tools/create-document.ts`

```typescript
import { generateUUID } from "@/lib/utils";
import { tool, type UIMessageStreamWriter } from "ai";
import { z } from "zod";
import type { Session } from "next-auth";
import { artifactKinds, documentHandlersByArtifactKind } from "@/lib/artifacts/server";
import type { ChatMessage } from "@/lib/types";

interface CreateDocumentProps {
  session: Session;
  dataStream: UIMessageStreamWriter<ChatMessage>;
}

export const createDocument = ({ session, dataStream }: CreateDocumentProps) =>
  tool({
    description:
      "Create a document for a writing or content creation activities. This tool will call other functions that will generate the contents of the document based on the title and kind.",
    inputSchema: z.object({
      title: z.string(),
      kind: z.enum(artifactKinds),
    }),
    execute: async ({ title, kind }) => {
      const id = generateUUID();

      dataStream.write({
        type: "data-kind",
        data: kind,
        transient: true,
      });

      dataStream.write({
        type: "data-id",
        data: id,
        transient: true,
      });

      dataStream.write({
        type: "data-title",
        data: title,
        transient: true,
      });

      dataStream.write({
        type: "data-clear",
        data: null,
        transient: true,
      });

      const documentHandler = documentHandlersByArtifactKind.find(
        (documentHandlerByArtifactKind) => documentHandlerByArtifactKind.kind === kind
      );

      if (!documentHandler) {
        throw new Error(`No document handler found for kind: ${kind}`);
      }

      await documentHandler.onCreateDocument({
        id,
        title,
        dataStream,
        session,
      });

      dataStream.write({ type: "data-finish", data: null, transient: true });

      return {
        id,
        title,
        kind,
        content: "A document was created and is now visible to the user.",
      };
    },
  });
```

### 3.5.2 Document Update Tool

**File to Create**: `src/nextjs/src/lib/ai/tools/update-document.ts`

```typescript
import { tool, type UIMessageStreamWriter } from "ai";
import type { Session } from "next-auth";
import { z } from "zod";
import { getDocumentById } from "@/lib/db/queries";
import { documentHandlersByArtifactKind } from "@/lib/artifacts/server";
import type { ChatMessage } from "@/lib/types";

interface UpdateDocumentProps {
  session: Session;
  dataStream: UIMessageStreamWriter<ChatMessage>;
}

export const updateDocument = ({ session, dataStream }: UpdateDocumentProps) =>
  tool({
    description: "Update a document with the given description.",
    inputSchema: z.object({
      id: z.string().describe("The ID of the document to update"),
      description: z.string().describe("The description of changes that need to be made"),
    }),
    execute: async ({ id, description }) => {
      const document = await getDocumentById({ id });

      if (!document) {
        return {
          error: "Document not found",
        };
      }

      dataStream.write({
        type: "data-clear",
        data: null,
        transient: true,
      });

      const documentHandler = documentHandlersByArtifactKind.find(
        (documentHandlerByArtifactKind) => documentHandlerByArtifactKind.kind === document.kind
      );

      if (!documentHandler) {
        throw new Error(`No document handler found for kind: ${document.kind}`);
      }

      await documentHandler.onUpdateDocument({
        document,
        description,
        dataStream,
        session,
      });

      dataStream.write({ type: "data-finish", data: null, transient: true });

      return {
        id,
        title: document.title,
        kind: document.kind,
        content: "The document has been updated successfully.",
      };
    },
  });
```

### 3.5.3 AI Suggestions Tool

**File to Create**: `src/nextjs/src/lib/ai/tools/request-suggestions.ts`

```typescript
import { z } from "zod";
import type { Session } from "next-auth";
import { streamObject, tool, type UIMessageStreamWriter } from "ai";
import { getDocumentById, saveSuggestions } from "@/lib/db/queries";
import type { Suggestion } from "@/lib/db/schema";
import { generateUUID } from "@/lib/utils";
import { myProvider } from "../providers";
import type { ChatMessage } from "@/lib/types";

interface RequestSuggestionsProps {
  session: Session;
  dataStream: UIMessageStreamWriter<ChatMessage>;
}

export const requestSuggestions = ({ session, dataStream }: RequestSuggestionsProps) =>
  tool({
    description: "Request suggestions for a document",
    inputSchema: z.object({
      documentId: z.string().describe("The ID of the document to request edits"),
    }),
    execute: async ({ documentId }) => {
      const document = await getDocumentById({ id: documentId });

      if (!document || !document.content) {
        return {
          error: "Document not found",
        };
      }

      const suggestions: Array<Omit<Suggestion, "userId" | "createdAt" | "documentCreatedAt">> = [];

      const { elementStream } = streamObject({
        model: myProvider.languageModel("artifact-model"),
        system:
          "You are a help writing assistant. Given a piece of writing, please offer suggestions to improve the piece of writing and describe the change. It is very important for the edits to contain full sentences instead of just words. Max 5 suggestions.",
        prompt: document.content,
        output: "array",
        schema: z.object({
          originalSentence: z.string().describe("The original sentence"),
          suggestedSentence: z.string().describe("The suggested sentence"),
          description: z.string().describe("The description of the suggestion"),
        }),
      });

      for await (const element of elementStream) {
        // @ts-ignore todo: fix type
        const suggestion: Suggestion = {
          originalText: element.originalSentence,
          suggestedText: element.suggestedSentence,
          description: element.description,
          id: generateUUID(),
          documentId: documentId,
          isResolved: false,
        };

        dataStream.write({
          type: "data-suggestion",
          data: suggestion,
          transient: true,
        });

        suggestions.push(suggestion);
      }

      if (session.user?.id) {
        const userId = session.user.id;

        await saveSuggestions({
          suggestions: suggestions.map((suggestion) => ({
            ...suggestion,
            userId,
            createdAt: new Date(),
            documentCreatedAt: document.createdAt,
          })),
        });
      }

      return {
        id: documentId,
        title: document.title,
        kind: document.kind,
        message: "Suggestions have been added to the document",
      };
    },
  });
```

### 3.5.4 Weather Tool (Optional)

**File to Create**: `src/nextjs/src/lib/ai/tools/get-weather.ts`

```typescript
import { tool } from "ai";
import { z } from "zod";

export const getWeather = tool({
  description: "Get the current weather for a location",
  inputSchema: z.object({
    location: z.string().describe("The location to get weather for"),
  }),
  execute: async ({ location }) => {
    // Implementation would depend on weather API integration
    return {
      location,
      temperature: "22°C",
      condition: "Sunny",
      humidity: "65%",
    };
  },
});
```

## Phase 3.6: Integration Points

### 3.6.1 Dependencies Required

```json
{
  "dependencies": {
    "@ai-sdk/gateway": "^1.0.15",
    "@ai-sdk/provider": "2.0.0",
    "@ai-sdk/react": "2.0.26",
    "@ai-sdk/xai": "2.0.13",
    "ai": "5.0.26",
    "zod": "^3.22.0"
  }
}
```

### 3.6.2 Environment Variables

```env
# AI Gateway (already configured)
AI_GATEWAY_API_KEY=your_gateway_key

# Environment detection
NODE_ENV=development|production|test
```

### 3.6.3 File Structure

```
src/nextjs/src/lib/ai/
├── models.ts              # Model definitions
├── providers.ts           # AI provider configuration
├── prompts.ts             # Prompt templates
├── entitlements.ts        # User entitlements
├── models.mock.ts         # Mock models for testing
├── models.test.ts         # Model tests
└── tools/
    ├── create-document.ts # Document creation tool
    ├── update-document.ts # Document update tool
    ├── request-suggestions.ts # AI suggestions tool
    └── get-weather.ts     # Weather tool (optional)
```

## Phase 3.7: Implementation Steps ✅ COMPLETED

### Step 1: Create AI Configuration Files ✅ COMPLETED

1. ✅ Create `src/lib/ai/models.ts` with model definitions
2. ✅ Create `src/lib/ai/providers.ts` with provider setup
3. ✅ Create `src/lib/ai/prompts.ts` with prompt templates
4. ✅ Create `src/lib/ai/entitlements.ts` with user entitlements

### Step 2: Create AI Tools ✅ COMPLETED

1. ✅ Create `src/lib/ai/tools/` directory
2. ✅ Implement document creation and update tools
3. ✅ Implement AI suggestions tool
4. ✅ Add optional tools (weather, etc.)

### Step 3: Create Test Files ✅ COMPLETED

1. ✅ Create `src/lib/ai/models.mock.ts` for testing
2. ✅ Create `src/lib/ai/models.test.ts` for unit tests

### Step 4: Create Supporting Files ✅ COMPLETED

1. ✅ Create `src/lib/constants.ts` for environment detection
2. ✅ Copy `src/lib/artifacts/` system for document handling
3. ✅ Set up complete AI folder structure

### Step 5: Next Steps (Pending)

1. ⚠️ Fix import paths in copied AI files
2. ⚠️ Adapt entitlements to Futura's user system
3. ⚠️ Create missing type definitions
4. ⚠️ Test AI functionality

## Phase 3.8: Key Features

### 3.8.1 AI Model Features

- **Multimodal Support**: Vision and text capabilities
- **Chain-of-Thought Reasoning**: Advanced reasoning with Grok 3 Mini
- **Model Selection**: Users can choose between different models
- **Test Environment**: Mock models for development

### 3.8.2 Document Management

- **Real-time Creation**: Live document generation
- **Artifact System**: Side-by-side document editing
- **AI Suggestions**: Intelligent writing improvements
- **Multiple Formats**: Text, code, and spreadsheet support

### 3.8.3 User Experience

- **Rate Limiting**: Daily message limits
- **Geographic Context**: Location-aware responses
- **Streaming Responses**: Real-time AI interactions
- **Error Handling**: Graceful error management

## ✅ Phase 3 Summary: What We Accomplished

### Files Successfully Copied and Integrated:

1. **Core AI Configuration** ✅

   - `models.ts` - AI model definitions (Grok Vision, Grok Reasoning)
   - `providers.ts` - Vercel AI Gateway + XAI model setup
   - `prompts.ts` - Complete prompt system with artifacts support
   - `entitlements.ts` - User rate limiting and model access

2. **AI Tools System** ✅

   - `create-document.ts` - Real-time document creation
   - `update-document.ts` - Document editing and updates
   - `request-suggestions.ts` - AI-powered writing suggestions
   - `get-weather.ts` - Optional weather information tool

3. **Testing Infrastructure** ✅

   - `models.mock.ts` - Mock models for development/testing
   - `models.test.ts` - Unit tests for AI models

4. **Supporting Systems** ✅
   - `constants.ts` - Environment detection (production/development/test)
   - `artifacts/server.ts` - Document artifact system
   - Complete folder structure in `src/nextjs/src/lib/`

### Dependencies Already Available:

- ✅ `@ai-sdk/gateway` - AI Gateway integration
- ✅ `@ai-sdk/provider` - Provider abstraction
- ✅ `@ai-sdk/react` - React hooks for AI
- ✅ `@ai-sdk/xai` - XAI model support
- ✅ `ai` - Core AI SDK
- ✅ `zod` - Schema validation

### Environment Variables Ready:

- ✅ `AI_GATEWAY_API_KEY` - Already configured
- ✅ `NODE_ENV` - Environment detection

## Next Steps

After completing Phase 3, we'll move to Phase 4 (Core Components Import) to integrate the AI configuration with the UI components and create the chat interface.

**Immediate Next Steps:**

1. ⚠️ Fix import paths in copied AI files
2. ⚠️ Adapt entitlements to Futura's user system
3. ⚠️ Create missing type definitions
4. ⚠️ Test AI functionality

The AI configuration provides the foundation for:

- Intelligent chat responses
- Document creation and editing
- AI-powered suggestions
- Multimodal interactions
- User entitlement management
