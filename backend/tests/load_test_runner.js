// Simple load test script using native Node.js
// Run with: node load_test_runner.js

const http = require('http');

const BASE_URL = process.env.BASE_URL || 'http://localhost:3000';
const DURATION_MS = parseInt(process.env.DURATION_MS) || 60000; // 1 minute default
const CONCURRENT_USERS = parseInt(process.env.CONCURRENT_USERS) || 100;

const symbols = ['btcusdt', 'ethusdt', 'bnbusdt'];

let totalRequests = 0;
let failedRequests = 0;
const latencies = [];
let running = true;

function makeRequest(endpoint, method = 'GET') {
  return new Promise((resolve) => {
    const start = Date.now();
    const req = http.request(`${BASE_URL}${endpoint}`, { method }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => {
        const latency = Date.now() - start;
        totalRequests++;
        latencies.push(latency);
        if (res.statusCode >= 400) {
          failedRequests++;
        }
        resolve({ status: res.statusCode, latency });
      });
    });
    
    req.on('error', () => {
      totalRequests++;
      failedRequests++;
      resolve({ status: 0, latency: 0 });
    });
    
    req.end();
  });
}

async function userLoop(userId) {
  const symbol = symbols[userId % symbols.length];
  
  while (running) {
    await Promise.all([
      makeRequest('/health'),
      makeRequest('/ready'),
      makeRequest('/metrics'),
      makeRequest(`/api/candles?symbol=${symbol}&interval=1m&limit=100`),
    ]);
    
    await new Promise(r => setTimeout(r, 500));
  }
}

async function runLoadTest() {
  console.log(`Starting load test: ${CONCURRENT_USERS} users for ${DURATION_MS}ms`);
  console.log(`Target: ${BASE_URL}`);
  console.log('---');
  
  const startTime = Date.now();
  const users = [];
  
  // Start concurrent users
  for (let i = 0; i < CONCURRENT_USERS; i++) {
    users.push(userLoop(i));
  }
  
  // Run for specified duration
  await new Promise(r => setTimeout(r, DURATION_MS));
  
  // Stop all users
  running = false;
  await Promise.all(users);
  
  const duration = (Date.now() - startTime) / 1000;
  
  // Calculate metrics
  latencies.sort((a, b) => a - b);
  const p50 = latencies[Math.floor(latencies.length * 0.50)] || 0;
  const p95 = latencies[Math.floor(latencies.length * 0.95)] || 0;
  const p99 = latencies[Math.floor(latencies.length * 0.99)] || 0;
  const avgLatency = latencies.reduce((a, b) => a + b, 0) / latencies.length || 0;
  const errorRate = (failedRequests / totalRequests * 100).toFixed(2);
  
  console.log('\n============================================================');
  console.log('                    LOAD TEST RESULTS');
  console.log('============================================================');
  console.log(`\nTest Duration: ${duration.toFixed(1)}s`);
  console.log(`\nRequests:`);
  console.log(`  - Total: ${totalRequests}`);
  console.log(`  - Failed: ${failedRequests}`);
  console.log(`  - Error Rate: ${errorRate}%`);
  console.log(`  - Throughput: ${(totalRequests / duration).toFixed(1)} req/s`);
  console.log(`\nLatency (ms):`);
  console.log(`  - Avg: ${avgLatency.toFixed(2)}`);
  console.log(`  - p50: ${p50.toFixed(2)}`);
  console.log(`  - p95: ${p95.toFixed(2)}`);
  console.log(`  - p99: ${p99.toFixed(2)}`);
  console.log('\nSuccess Criteria Check:');
  console.log(`  - p99 < 500ms: ${p99 < 500 ? '✓ PASS' : '✗ FAIL'}`);
  console.log(`  - Error rate < 0.1%: ${parseFloat(errorRate) < 0.1 ? '✓ PASS' : '✗ FAIL'}`);
  console.log('\n============================================================');
}

runLoadTest().catch(console.error);