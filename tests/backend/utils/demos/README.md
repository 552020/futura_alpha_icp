# ICP Backend Test Framework - Demos

This folder contains practical demonstrations of how to use the ICP Backend Test Framework for memory creation, retrieval, and bulk operations.

## 🚀 **Quick Start**

### **Run a Simple Demo:**
```bash
# Simple memory creation and retrieval
node demo-simple-memory.mjs

# Comprehensive memory operations
node demo-memory-creation.mjs

# Bulk operations and performance testing
node demo-bulk-operations.mjs
```

## 📁 **Demo Files**

### **1. `demo-simple-memory.mjs`**
**Perfect for beginners** - Shows the basics of memory creation and retrieval.

**What it demonstrates:**
- ✅ Creating a test actor with proper certificate handling
- ✅ Creating a test capsule for memory storage
- ✅ Creating inline memories with real content
- ✅ Retrieving and verifying memory content
- ✅ Cleaning up test data

**Run it:**
```bash
node demo-simple-memory.mjs
```

### **2. `demo-memory-creation.mjs`**
**Comprehensive demo** - Shows advanced memory operations with performance testing.

**What it demonstrates:**
- ✅ Inline memory creation and retrieval
- ✅ Blob memory creation and retrieval
- ✅ Content integrity verification
- ✅ Performance comparison between storage types
- ✅ Real data creation and validation

**Run it:**
```bash
node demo-memory-creation.mjs
```

### **3. `demo-bulk-operations.mjs`**
**Bulk operations demo** - Shows how to efficiently manage multiple memories.

**What it demonstrates:**
- ✅ Bulk memory creation
- ✅ Bulk memory deletion
- ✅ Performance comparison (individual vs bulk)
- ✅ Error handling with invalid data
- ✅ Scalability testing

**Run it:**
```bash
node demo-bulk-operations.mjs
```

## 🎯 **What Each Demo Teaches**

### **Simple Memory Demo:**
- **Basic workflow** - Actor → Capsule → Memory → Retrieve → Cleanup
- **Certificate handling** - Automatic `fetchRootKey()` for local replica
- **Content verification** - Ensuring data integrity
- **Error handling** - Graceful failure management

### **Memory Creation Demo:**
- **Storage types** - Inline vs Blob vs External storage
- **Content types** - Text, images, documents, audio, video
- **Performance analysis** - Creation and retrieval timing
- **Data validation** - Content integrity and metadata verification

### **Bulk Operations Demo:**
- **Efficiency** - Bulk operations vs individual operations
- **Scalability** - Performance with different dataset sizes
- **Error resilience** - Handling invalid data gracefully
- **Resource management** - Proper cleanup and memory management

## 🔧 **Prerequisites**

### **Environment Setup:**
```bash
# Ensure local replica is running
dfx start

# Deploy the backend canister
dfx deploy backend

# Set environment variables
export BACKEND_CANISTER_ID="uxrrr-q7777-77774-qaaaq-cai"
export IC_HOST="http://127.0.0.1:4943"
```

### **Dependencies:**
- Node.js (v18+)
- ICP local replica running
- Backend canister deployed
- Test framework utilities

## 📊 **Expected Output**

### **Simple Memory Demo:**
```
🚀 Simple Memory Demo
========================================
1️⃣ Creating test actor and capsule...
✅ Capsule ready: uxrrr-q7777-77774-qaaaq-cai
2️⃣ Preparing test content...
📝 Content: "Hello, this is a simple test memory!"
📊 Size: 1.0 KB
3️⃣ Creating asset metadata...
✅ Asset metadata created
4️⃣ Creating memory...
✅ Memory created: memory_1234567890
⏱️  Time: 245ms
5️⃣ Retrieving memory...
✅ Memory retrieved successfully!
⏱️  Time: 89ms
6️⃣ Memory Details:
  🆔 ID: memory_1234567890
  📝 Title: simple_test_memory
  📄 Type: Note
  📅 Created: 2024-01-15T10:30:45.123Z
  📦 Inline Assets: 1
  🗂️  Blob Assets: 0
✅ Content verified successfully!
7️⃣ Cleaning up...
✅ Memory deleted
🎉 Demo completed successfully!
```

### **Memory Creation Demo:**
```
🚀 Memory Creation and Retrieval Demo
Using the ICP Backend Test Framework
============================================================
🧪 Demo 1: Inline Memory Creation and Retrieval
============================================================
📝 Content: "Hello, this is a real inline memory stored directly in the memory struct!"
📊 Content size: 1.2 KB
🔐 Content hash: a1b2c3d4e5f6...
📋 Asset metadata created:
{
  "base": {
    "name": "demo_inline_memory",
    "bytes": 1024,
    "mime_type": "text/plain",
    "sha256": "a1b2c3d4e5f6...",
    "description": "Original"
  }
}
🚀 Creating inline memory...
✅ Memory created: memory_1234567890
⏱️  Creation time: 267ms
🔍 Retrieving memory...
✅ Memory retrieved successfully!
⏱️  Retrieval time: 95ms
📄 Memory Details:
  🆔 ID: memory_1234567890
  📝 Title: demo_inline_memory
  📄 Content Type: text/plain
  📅 Created At: 2024-01-15T10:30:45.123Z
  🏷️  Tags: test, demo
  👤 Created By: 2vxsx-fae
🔍 Content Verification:
  📦 Inline Assets: 1
  🗂️  Blob Internal Assets: 0
  🌐 Blob External Assets: 0
✅ Content integrity verified!
📝 Retrieved content: "Hello, this is a real inline memory stored directly in the memory struct!"
🧹 Cleaning up...
✅ Memory deleted
🎉 Inline memory demo completed successfully!
```

### **Bulk Operations Demo:**
```
🚀 Bulk Memory Operations Demo
Using the ICP Backend Test Framework
============================================================
🧪 Demo 1: Bulk Memory Creation
==================================================
📝 Creating 5 test memories...
✅ Created 5 memories in 1.2s
📊 Average time per memory: 240ms
🆔 Memory IDs: memory_1, memory_2, memory_3, memory_4, memory_5
🧹 Cleaning up...
✅ All memories deleted
🎉 Bulk creation demo completed!

🧪 Demo 2: Bulk Memory Deletion
==================================================
📝 Creating 10 test memories...
✅ Created 10 memories
🗑️  Testing bulk deletion...
✅ Bulk deletion completed in 156ms
📊 Deleted count: 10
❌ Failed count: 0
💬 Message: Successfully deleted 10 memories
🎉 Bulk deletion demo completed!

🧪 Demo 3: Performance Comparison
==================================================
📊 Testing with 5 memories:
  ✅ Created 5 memories
  🗑️  Testing individual deletion...
    ⏱️  Individual deletion: 1.2s
  🗑️  Testing bulk deletion...
    ⏱️  Bulk deletion: 156ms
    📈 Speedup: 7.69x faster
```

## 🎯 **Key Learning Points**

### **1. Certificate Verification:**
- ✅ **Automatic handling** - Framework handles `fetchRootKey()` automatically
- ✅ **No manual configuration** - Works out of the box
- ✅ **Expert validated** - Multiple ICP experts confirmed the solution

### **2. Real Data Creation:**
- ✅ **Actual content** - Not fake IDs or placeholder data
- ✅ **Content integrity** - Full verification of stored content
- ✅ **Metadata validation** - Proper asset metadata structures

### **3. Performance Insights:**
- ✅ **Bulk operations** - Significantly faster than individual operations
- ✅ **Storage types** - Different performance characteristics
- ✅ **Scalability** - Performance scales with dataset size

### **4. Error Handling:**
- ✅ **Graceful failures** - Proper error messages and recovery
- ✅ **Invalid data** - Handles non-existent memory IDs
- ✅ **Empty operations** - Handles edge cases properly

## 🚨 **Troubleshooting**

### **Common Issues:**

#### **Certificate Verification Error:**
```
❌ Certificate verification error: "Signature verification failed"
```
**Solution:** ✅ **SOLVED** - Framework includes proper `fetchRootKey()` handling

#### **Type Mismatch Error:**
```
❌ Invalid opt vec nat8 argument: {...}
```
**Solution:** Ensure content bytes are wrapped in arrays: `[contentBytes]`

#### **Connection Error:**
```
❌ Failed to create test agent: ECONNREFUSED
```
**Solution:** Ensure local replica is running (`dfx start`)

### **Debug Mode:**
```bash
# Enable debug logging
export DEBUG="true"
node demo-simple-memory.mjs
```

## 📚 **Next Steps**

After running these demos, you'll understand:
1. **How to create and retrieve memories** using the framework
2. **How bulk operations work** and their performance benefits
3. **How to handle errors** gracefully
4. **How to measure performance** and optimize operations
5. **How to clean up** test data properly

## 🎉 **Success Metrics**

- ✅ **All demos run successfully** - No certificate verification errors
- ✅ **Real data creation** - Actual content stored and retrieved
- ✅ **Performance measurement** - Timing and throughput analysis
- ✅ **Error handling** - Graceful failure management
- ✅ **Cleanup** - Proper test data cleanup

---

**These demos provide everything you need to understand and use the ICP Backend Test Framework effectively!** 🚀
