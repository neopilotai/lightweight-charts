import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Counter, Trend } from 'k6/metrics';

export const options = {
  scenarios: {
    // Ramp up: 0 -> 100 users in 1min
    ramp_up: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '1m', target: 100 },
      ],
      gracefulRampDown: '30s',
    },
    // Hold: 100 users for 5min
    hold_100: {
      executor: 'constant-vus',
      vus: 100,
      duration: '5m',
      startTime: '1m', // Start after ramp_up
    },
    // Ramp up: 100 -> 1000 users in 5min
    ramp_to_1000: {
      executor: 'ramping-vus',
      startVUs: 100,
      stages: [
        { duration: '5m', target: 1000 },
      ],
      gracefulRampDown: '30s',
      startTime: '6m', // Start after hold_100
    },
    // Hold: 1000 users for 10min
    hold_1000: {
      executor: 'constant-vus',
      vus: 1000,
      duration: '10m',
      startTime: '11m', // Start after ramp_to_1000
    },
    // Ramp down: 1000 -> 0 users in 5min
    ramp_down: {
      executor: 'ramping-vus',
      startVUs: 1000,
      stages: [
        { duration: '5m', target: 0 },
      ],
      gracefulRampDown: '30s',
      startTime: '21m', // Start after hold_1000
    },
  },
  thresholds: {
    // Success criteria from checklist
    http_req_duration: ['p(50)<500', 'p(95)<1000', 'p(99)<2000'],
    http_req_failed: ['rate<0.001'], // Error rate < 0.1%
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';
const SYMBOLS = ['btcusdt', 'ethusdt', 'bnbusdt'];
const WS_URL = __ENV.WS_URL || 'ws://localhost:3000/ws';

const errorRate = new Rate('errors');
const requestsCount = new Counter('requests');
const messageCount = new Counter('messages_received');
const latencyTrend = new Trend('latency');

export default function () {
  const symbol = SYMBOLS[Math.floor(Math.random() * SYMBOLS.length)];
  
  group('HTTP Endpoints', () => {
    // Test /health endpoint
    group('Health Check', () => {
      const start = Date.now();
      const res = http.get(`${BASE_URL}/health`);
      latencyTrend.add(Date.now() - start);
      requestsCount.add(1);
      
      const success = check(res, {
        'health returns 200': (r) => r.status === 200,
        'health has status': (r) => JSON.parse(r.body).status === 'healthy',
      });
      errorRate.add(!success);
    });

    // Test /ready endpoint
    group('Readiness Check', () => {
      const start = Date.now();
      const res = http.get(`${BASE_URL}/ready`);
      latencyTrend.add(Date.now() - start);
      requestsCount.add(1);
      
      const success = check(res, {
        'ready returns 200 or 503': (r) => r.status === 200 || r.status === 503,
      });
      errorRate.add(!success);
    });

    // Test /metrics endpoint
    group('Metrics Endpoint', () => {
      const start = Date.now();
      const res = http.get(`${BASE_URL}/metrics`);
      latencyTrend.add(Date.now() - start);
      requestsCount.add(1);
      
      const success = check(res, {
        'metrics returns 200': (r) => r.status === 200,
        'metrics has content': (r) => r.body.length > 0,
      });
      errorRate.add(!success);
    });

    // Test /api/candles endpoint
    group('Candles API', () => {
      const start = Date.now();
      const res = http.get(`${BASE_URL}/api/candles?symbol=${symbol}&interval=1m&limit=100`);
      latencyTrend.add(Date.now() - start);
      requestsCount.add(1);
      
      const success = check(res, {
        'candles returns 200': (r) => r.status === 200,
        'candles has data': (r) => {
          try {
            const body = JSON.parse(r.body);
            return body.candles && body.candles.length > 0;
          } catch (e) {
            return false;
          }
        },
      });
      errorRate.add(!success);
    });
  });

  // Small delay between iterations
  sleep(0.5);
}

export function handleSummary(data) {
  const duration = data.state.testDurationMs / 1000;
  const totalRequests = data.metrics.http_reqs.values.count;
  const failedRequests = data.metrics.http_req_failed.values.passes;
  const errorRatePercent = (failedRequests / totalRequests * 100).toFixed(2);
  
  // Calculate latency percentiles
  const p50 = data.metrics.http_req_duration.values['p(50)'];
  const p95 = data.metrics.http_req_duration.values['p(95)'];
  const p99 = data.metrics.http_req_duration.values['p(99)'];

  return {
    stdout: `
============================================================
                    LOAD TEST RESULTS
============================================================

Test Duration: ${duration.toFixed(1)}s

Requests:
  - Total: ${totalRequests}
  - Failed: ${failedRequests}
  - Error Rate: ${errorRatePercent}%

Latency (ms):
  - p50: ${p50.toFixed(2)}
  - p95: ${p95.toFixed(2)}
  - p99: ${p99.toFixed(2)}

Success Criteria Check:
  - p99 < 2000ms: ${p99 < 2000 ? '✓ PASS' : '✗ FAIL'}
  - Error rate < 0.1%: ${parseFloat(errorRatePercent) < 0.1 ? '✓ PASS' : '✗ FAIL'}

============================================================
                    TEST COMPLETE
============================================================
    `,
  };
}