# Contributing to FlatMesh

Thank you for your interest in contributing to **flatmesh-rs**!

## Engineering Guidelines & Invariants

1. **Zero-Allocation Hot Path**: The core parsing and memory-mapping read paths must never allocate on the heap. All layout parsing must be verified with zero heap allocations.
2. **Alignment & Cache Line Sympathy**: Headers and lump tables must adhere to 32-byte and 64-byte alignments. Unaligned reads or unverified pointer conversions are strictly forbidden.
3. **Safety Explanations**: Every `unsafe` block must include a detailed `// SAFETY:` rationale explaining pointer provenance, alignment, and bounds validation.
4. **Testing Triad**: All PRs must pass `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`, and our property-based fuzzing and chaos suites.

## Pull Request Process

1. Target the `dev` branch for all features and fixes.
2. Ensure conventional commits (e.g. `feat(core): implement meshlet bounding cone packing`).
3. Run `cargo fmt` and `cargo clippy --all-targets --all-features -- -D warnings` locally before submitting.
