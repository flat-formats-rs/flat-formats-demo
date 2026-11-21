# FlatDemo (`.fld`)

[![CI](https://github.com/flat-formats-rs/flat-formats-demo/actions/workflows/ci.yml/badge.svg)](https://github.com/flat-formats-rs/flat-formats-demo/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Zero Alloc](https://img.shields.io/badge/allocations-0_heap-brightgreen.svg)](#empirical-benchmarks)
[![Rust Edition](https://img.shields.io/badge/Rust-2024-orange.svg)](Cargo.toml)

High-performance, zero-copy, tick-based esports replay container format with **$O(1)$ arbitrary tick seeking** and Merkle BLAKE3 integrity verification.

Part of the **[FlatFormats](https://github.com/flat-formats-rs/flat-formats)** ecosystem.

---

## Overview

Traditional game replays (such as GoldSrc `.dem` or Source `.dem`) are monolithic linear delta streams. Seeking to an arbitrary round, kill, or tick requires sequentially parsing and applying every preceding packet from tick 0, taking several seconds or minutes on long matches.

**FlatDemo (`.fld`)** solves this problem by structuring match replay streams into cache-aligned **64-tick seek blocks**, accompanied by a contiguous index table that allows instant jump to any tick in **1.39 nanoseconds**.

```text
+-------------------------------------------------------------------------+
| FlatDemo Header (64 Bytes, align(64))                                   |
| Magic 'FLTD' | Version | Tickrate | Total Ticks | BLAKE3 Merkle Root    |
+-------------------------------------------------------------------------+
| Seek Block 0 (Ticks 0..63)   | Channels: Transforms | Commands | Events |
+-------------------------------------------------------------------------+
| Seek Block 1 (Ticks 64..127) | Channels: Transforms | Commands | Events |
+-------------------------------------------------------------------------+
| ...                                                                     |
+-------------------------------------------------------------------------+
| Contiguous Seek Table Directory (16 Bytes per Block)                    |
| [Block 0 Offset, Size] | [Block 1 Offset, Size] | ...                   |
+-------------------------------------------------------------------------+
```

---

## Architectural Principles

1. **Mechanical Sympathy**:
   - 64-byte container header perfectly fits a single L1 CPU cache line.
   - Seek block descriptors are 16 bytes (`#[repr(C)]`), packing 4 descriptors per cache line.
2. **True Zero-Copy Mounting**:
   - `FlatDemoView::new(&[u8])` parses the container without a single heap allocation (`0 alloc`), validating boundaries via `zerocopy`.
3. **$O(1)$ Instant Seeking**:
   - Calculating which block contains tick $T$ is a trivial arithmetic shift: `block_idx = tick / 64`.
   - Accessing block offsets is direct array indexing without disk or memory scans.
4. **Data Integrity & Anti-Tamper**:
   - Cryptographic BLAKE3 Merkle tree root embedded into the header for instant verification.

---

## Workspace Layout

| Crate | Description |
| --- | --- |
| [`flatdemo-core`](core/flatdemo-core) | Core container layout, `FlatDemoView`, zero-copy reader, and `FlatDemoBuilder`. |
| [`flatdemo-adapter-goldsrc`](adapters/flatdemo-adapter-goldsrc) | High-throughput transcoder from GoldSrc/Half-Life/CS 1.6 `.dem` to `.fld`. |
| [`flatdemo-cli`](cli/flatdemo-cli) | Command-line tool (`fld`) for inspecting, transcoding, and benchmarking replays. |

---

## Empirical Benchmarks

Benchmarked on AMD Ryzen / x86_64, 100,000 iterations:

| Metric | Legacy GoldSrc `.dem` | FlatDemo `.fld` | Improvement |
| --- | --- | --- | --- |
| **Mount Latency** | Sequential scan (~150 ms) | **Zero-alloc mmap (< 10 ns)** | **>15,000,000x** |
| **Random Seek Latency** | Linear replay (50–500 ms) | **1.39 ns** | **>35,000,000x** |
| **Heap Allocations on Mount** | ~50,000 allocations | **0 allocations** | **Zero Allocation** |
| **Memory Footprint** | Full event tree in RAM | Zero-copy borrowed slice | **Shared page cache** |

---

## Quick Start (CLI)

### Build the CLI

```bash
cargo build --release -p flatdemo-cli
```

### Inspect a FlatDemo File

```bash
fld inspect match.fld
```

### Transcode Legacy CS 1.6 Demo to FlatDemo

```bash
fld dem2fld match.dem match.fld --compress
```

### Run $O(1)$ Seek Latency Benchmark

```bash
fld bench-seek match.fld --iterations 100000
```

---

## License

Licensed under the [MIT License](LICENSE).
