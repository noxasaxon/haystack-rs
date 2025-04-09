# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Python Codebase (Original)

### Build/Test/Lint Commands
- Install dependencies: `hatch shell`
- Run unit tests: `hatch run test:unit`
- Run specific test: `hatch run test:unit test/path/to/test.py::TestClass::test_method`
- Run integration tests: `hatch run test:integration`
- Run end-to-end tests: `hatch run test:e2e`
- Type checking: `hatch run test:types`
- Linting: `hatch run test:lint`
- Format code: `hatch run format`

### Code Style (Python)
- Line length: 120 characters max
- Include docstrings for public functions/methods
- Use Python type hints consistently
- Use absolute imports, sorted with isort
- Follow Python naming conventions (snake_case for variables/functions)
- Use the Haystack logger (`from haystack import logging`)

## Rust Codebase (Port)

### Setup
- Create Rust code in `rust/` directory
- Maintain 100% functional equivalence with Python code
- Port one module/component at a time, using e2e/integration tests to verify

### Build/Test Commands
- Build: `cargo build`
- Run tests: `cargo test`
- Run specific test: `cargo test test_name`
- Format code: `cargo fmt`
- Lint code: `cargo clippy`

### Port Testing Strategy
- Focus on e2e/integration tests as primary verification method
- For each module being ported, first understand its integration tests
- Ensure Rust implementation passes the same integration tests
- Scope tasks around testable units of functionality
- Use Python tests as a reference for creating equivalent Rust tests

### Code Style (Rust)
- Follow Rust naming conventions (snake_case for functions, CamelCase for types)
- Create proper Rust error types with context
- Maintain API compatibility with Python version
- Favor Rust idioms while preserving behavior