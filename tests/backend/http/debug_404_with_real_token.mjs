#!/usr/bin/env node

import { createTestCapsule, createTestMemoryWithImage, mintHttpToken } from '../utils/helpers/http-auth.js';
import { measureExecutionTime } from '../utils/helpers/timing.js';

async function debug404WithRealToken() {
    console.log('🔍 Debugging 404 errors with real token...\n');
    
    try {
        // 1. Create a test capsule
        const capsuleId = await createTestCapsule();
        console.log(`✅ Test capsule created: ${capsuleId}`);
        
        // 2. Create a memory with an inline image asset
        const memoryId = await createTestMemoryWithImage(capsuleId);
        console.log(`✅ Test memory created: ${memoryId}`);
        
        // 3. Mint a token for the memory
        const token = await mintHttpToken(memoryId, ['thumbnail'], []);
        console.log(`✅ HTTP token minted: ${token.substring(0, 50)}...`);
        
        // 4. Test HTTP request with the real token
        const canisterId = 'uxrrr-q7777-77774-qaaaq-cai';
        const url = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?token=${encodeURIComponent(token)}`;
        
        console.log(`\n🌐 Testing HTTP request:`);
        console.log(`  URL: ${url}`);
        
        const response = await fetch(url);
        console.log(`  Status: ${response.status} ${response.statusText}`);
        console.log(`  Headers:`, Object.fromEntries(response.headers.entries()));
        
        const body = await response.text();
        console.log(`  Body: ${body}`);
        
        if (response.status === 200) {
            console.log('✅ HTTP request successful!');
        } else {
            console.log('❌ HTTP request failed');
        }
        
    } catch (error) {
        console.error('❌ Error during test:', error);
    }
}

// Run the test
debug404WithRealToken().catch(console.error);
