# @affectively/wasm-analytics-engine

## Purpose, architecture, and failure boundaries

The purpose of this architecture is bounded analytics in WASM. Typed input and output schemas, memory limits, and deterministic operations form the API contract. Malformed data, allocation failure, non-finite output, or unsupported modules reject. Treat modules and datasets as untrusted; tests do not establish causal interpretation.

High-performance WebAssembly analytics engine written in Rust.

[![npm](https://img.shields.io/npm/v/@affectively/wasm-analytics-engine.svg)](https://www.npmjs.com/package/@affectively/wasm-analytics-engine)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **Metrics Computation** - Calculate analytics metrics at WASM speed
- **Event Processing** - Process and transform event streams
- **Aggregations** - Efficient count, sum, average operations
- **Funnel Analysis** - Compute conversion funnels

## Installation

```bash
npm install @affectively/wasm-analytics-engine
```text

## Quick Start

```typescript
import init, { process_events, compute_funnel, aggregate_metrics } from '@affectively/wasm-analytics-engine';

await init();

const processed = process_events(rawEvents);
const funnel = compute_funnel(events, ['signup', 'verify', 'purchase']);
const metrics = aggregate_metrics(events, ['pageviews', 'sessions']);
```

## License

MIT License

---

Made with ️ by [AFFECTIVELY](https://affectively.ai)
