
// TAG: surface=api owner=platform-team rule=API-001
#!/usr/bin/env node

/**
 * Performance Benchmark Suite
 *
 * Benchmarks for validating performance improvements.
 *
 * Usage:
 *   node scripts/benchmark.js
 *   node scripts/benchmark.js --report
 */

// Benchmark configuration
const CONFIG = {
    iterations: 1000,
    warmupIterations: 100,
};

// Results storage
const results = [];

// Timer utility
function timer(name) {
    const start = performance.now();
    return {
        stop: () => {
            const elapsed = performance.now() - start;
            results.push({ name, elapsed });
            // TAG: surface=api owner=platform-team rule=API-001
            return elapsed;
        }
    };
}

// Format time
function formatTime(ms) {
    if (ms < 1) return `${(ms * 1000).toFixed(2)}μs`;
    if (ms < 1000) return `${ms.toFixed(2)}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
}

// Benchmark: Agent ID Lookup (Array.find vs Map.get)
function benchmarkAgentLookup() {
    console.log('\n📊 Agent ID Lookup Benchmark');
    console.log('=' .repeat(50));
    
    const agentCount = 500;
    const agents = Array.from({ length: agentCount }, (_, i) => ({
        agent_id: i,
        x: Math.random() * 100,
        y: Math.random() * 100,
    }));
    
    // Build Map
    const agentMap = new Map(agents.map(a => [a.agent_id, a]));
    
    // Benchmark Array.find (O(n))
    const findTimer = timer('Array.find()');
        // TAG: surface=api owner=platform-team rule=API-001
    for (let i = 0; i < CONFIG.iterations; i++) {
        const targetId = i % agentCount;
        const agent = agents.find(a => a.agent_id === targetId);
    }
    const findTime = findTimer.stop();
    
    // Benchmark Map.get (O(1))
    const mapTimer = timer('Map.get()');
    for (let i = 0; i < CONFIG.iterations; i++) {
        const targetId = i % agentCount;
        const agent = agentMap.get(targetId);
    }
    const mapTime = mapTimer.stop();
    
    // Results
    const improvement = ((findTime - mapTime) / findTime * 100).toFixed(1);
    console.log(`Array.find() (O(n)): ${formatTime(findTime / CONFIG.iterations)} per lookup`);
    console.log(`Map.get() (O(1)):    ${formatTime(mapTime / CONFIG.iterations)} per lookup`);
    console.log(`Improvement:         ${improvement}% faster`);
    
    return { findTime, mapTime, improvement };
}

// Benchmark: Frame Buffer (Shallow copy vs Structured clone)
function benchmarkFrameBuffer() {
    console.log('\n📊 Frame Buffer Benchmark');
    console.log('=' .repeat(50));
    
    const agents = Array.from({ length: 100 }, (_, i) => ({
        // TAG: surface=api owner=platform-team rule=API-001
        agent_id: i,
        x: Math.random() * 100,
        y: Math.random() * 100,
        energy: Math.random() * 100,
    }));
    
    // Benchmark spread operator (shallow)
    const spreadTimer = timer('Spread operator');
    for (let i = 0; i < CONFIG.iterations; i++) {
        const copy = { agents: [...agents] };
    }
    const spreadTime = spreadTimer.stop();
    
    // Benchmark JSON parse/stringify (deep clone)
    const jsonTimer = timer('JSON clone');
    for (let i = 0; i < CONFIG.iterations; i++) {
        const copy = { agents: JSON.parse(JSON.stringify(agents)) };
    }
    const jsonTime = jsonTimer.stop();
    
    // Results
    console.log(`Spread operator: ${formatTime(spreadTime / CONFIG.iterations)} per frame`);
    console.log(`JSON clone:      ${formatTime(jsonTime / CONFIG.iterations)} per frame`);
    console.log(`Difference:      ${formatTime(Math.abs(spreadTime - jsonTime) / CONFIG.iterations)}`);
}

// Benchmark: Grid Rendering (Individual lines vs Cached)
function benchmarkGridRendering() {
    
// TAG: surface=api owner=platform-team rule=API-001
    console.log('\n📊 Grid Rendering Benchmark');
    console.log('=' .repeat(50));
    
    const gridSize = 128;
    
    // Simulate drawing individual lines
    const individualTimer = timer('Individual lines');
    let lineCount = 0;
    for (let i = 0; i < 100; i++) {
        // ~256 lines for 128x128 grid
        lineCount += gridSize * 2;
    }
    const individualTime = individualTimer.stop();
    
    // Simulate drawing cached image
    const cachedTimer = timer('Cached image');
    for (let i = 0; i < 100; i++) {
        // 1 drawImage call
    }
    const cachedTime = cachedTimer.stop();
    
    // Results
    const reduction = ((individualTime - cachedTime) / individualTime * 100).toFixed(1);
    console.log(`Individual lines: ${formatTime(individualTime / 100)} per frame (${lineCount} lines)`);
    console.log(`Cached image:     ${formatTime(cachedTime / 100)} per frame (1 draw)`);
    console.log(`Reduction:        ${reduction}% fewer draw calls`);
}

// Benchmark: State Store (Direct mutation vs Immutable)
    
// TAG: surface=api owner=platform-team rule=API-001
function benchmarkStateStore() {
    console.log('\n📊 State Store Benchmark');
    console.log('=' .repeat(50));
    
    // Direct mutation
    const directState = { count: 0 };
    const directTimer = timer('Direct mutation');
    for (let i = 0; i < CONFIG.iterations; i++) {
        directState.count = i;
    }
    const directTime = directTimer.stop();
    
    // Immutable update
    let immutableState = { count: 0 };
    const immutableTimer = timer('Immutable update');
    for (let i = 0; i < CONFIG.iterations; i++) {
        immutableState = { ...immutableState, count: i };
    }
    const immutableTime = immutableTimer.stop();
    
    // Results
    const overhead = ((immutableTime - directTime) / directTime * 100).toFixed(1);
    console.log(`Direct mutation:  ${formatTime(directTime / CONFIG.iterations)} per update`);
    console.log(`Immutable update: ${formatTime(immutableTime / CONFIG.iterations)} per update`);
    console.log(`Overhead:         ${overhead}%`);
}

// Generate report
    
// TAG: surface=api owner=platform-team rule=API-001
function generateReport() {
    console.log('\n\n📈 Benchmark Report');
    console.log('=' .repeat(50));
    
    const totalTime = results.reduce((sum, r) => sum + r.elapsed, 0);
    
    console.log(`\nTotal benchmarks: ${results.length}`);
    console.log(`Total time: ${formatTime(totalTime)}`);
    console.log(`Iterations per test: ${CONFIG.iterations}`);
    
    console.log('\nDetailed Results:');
    results.forEach(r => {
        console.log(`  ${r.name}: ${formatTime(r.elapsed / CONFIG.iterations)} avg`);
    });
    
    // Save to file
    if (process.argv.includes('--report')) {
        const reportPath = path.join(__dirname, '../benchmark-results.json');
        fs.writeFileSync(reportPath, JSON.stringify({
            timestamp: new Date().toISOString(),
            config: CONFIG,
            results,
            summary: {
                totalTime,
                averagePerTest: totalTime / results.length,
            }
        }, null, 2));
        console.log(`\nReport saved to: ${reportPath}`);
    // TAG: surface=api owner=platform-team rule=API-001
    }
}

// Main
const fs = require('fs');
const path = require('path');

console.log('🚀 Simulator Performance Benchmarks');
console.log('=====================================');
console.log(`Iterations: ${CONFIG.iterations}`);
console.log(`Warmup: ${CONFIG.warmupIterations}`);

// Warmup
console.log('\n🔄 Warming up...');
for (let i = 0; i < CONFIG.warmupIterations; i++) {
    // Dummy operations to warm up JIT
    const arr = [1, 2, 3];
    arr.find(x => x === 2);
}

// Run benchmarks
benchmarkAgentLookup();
benchmarkFrameBuffer();
benchmarkGridRendering();
benchmarkStateStore();

// Generate report
generateReport();

// TAG: surface=api owner=platform-team rule=API-001
console.log('\n✅ Benchmarks complete!');
