# Haystack Rust

This is the Rust implementation of [Haystack](https://github.com/deepset-ai/haystack), a framework for building NLP pipelines.

## Structure

The code is organized as a Rust workspace with several packages:

- `haystack` - The main package that re-exports all the other packages
- `haystack-core` - Core components like Pipeline, Component, and error handling
- `haystack-dataclasses` - Data structures like Document, ByteStream, and ChatMessage

## Current Implementation Status

We have successfully implemented:

- Core data structures:
  - Document
  - ByteStream
  - ChatMessage
  - SparseEmbedding
  - Answer/ExtractedAnswer/GeneratedAnswer
  - StreamingChunk
  - State

- Core components framework:
  - Component trait
  - Socket system
  - Pipeline execution engine
  - Serialization/deserialization framework
  - Example components: TextSplitter and TextJoiner

## Running Tests

```bash
cargo test
```

## Development

### Requirements

- Rust 2021 edition (1.56.0 or later)
- Cargo

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Porting Strategy

The Rust implementation aims to maintain 100% functional equivalence with the Python code. The porting strategy is:

1. Port one module/component at a time
2. Use e2e/integration tests to verify functionality
3. Focus on the core components first, then expand to more specialized components
4. Maintain API compatibility with the Python version
5. Embrace Rust idioms while preserving behavior

## Code Style

- Follow Rust naming conventions (snake_case for functions, CamelCase for types)
- Create proper Rust error types with context
- Embrace Rust's ownership and borrowing system
- Use Rust idioms where appropriate (iterators, Option, Result, etc.)
- Write documentation comments for public APIs