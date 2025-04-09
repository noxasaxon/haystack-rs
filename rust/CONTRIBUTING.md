# Contributing to Haystack Rust

Thank you for your interest in contributing to Haystack Rust! This document provides guidelines and instructions for contributing to this project.

## Prerequisites

- Rust (1.56.0 or later)
- Cargo
- Basic knowledge of the Python Haystack framework
- Familiarity with Rust idioms and patterns

## Development Setup

1. Clone the repository:
   ```
   git clone https://github.com/deepset-ai/haystack.git
   cd haystack
   ```

2. Build the Rust code:
   ```
   cd rust
   cargo build
   ```

3. Run the tests:
   ```
   cargo test
   ```

## Porting Guidelines

When porting Python code to Rust, follow these guidelines:

1. **Understand the Python code first**: Make sure you understand what the Python code does, including edge cases and error handling.

2. **Maintain functional equivalence**: The Rust implementation should behave the same as the Python one.

3. **Use Rust idioms**: While maintaining functional equivalence, use idiomatic Rust patterns:
   - Use enums for variants instead of string constants
   - Use `Option` and `Result` appropriately
   - Leverage Rust's ownership system
   - Use iterators instead of explicit loops where appropriate

4. **Error handling**: Provide meaningful error messages with context.

5. **Documentation**: Document public APIs using rustdoc format.

6. **Testing**: Ensure each ported component has tests, relying on e2e/integration tests where appropriate.

## Code Style

- Use `cargo fmt` to format your code.
- Use `cargo clippy` to check for common mistakes and non-idiomatic code.
- Follow the Rust naming conventions:
  - `snake_case` for variables and functions
  - `CamelCase` for types and traits
  - `SCREAMING_SNAKE_CASE` for constants

## Pull Request Process

1. Make sure your code passes all tests and linting.
2. Update documentation if necessary.
3. Create a pull request with a clear description of the changes.
4. Wait for review and address any feedback.

## License

By contributing to this project, you agree that your contributions will be licensed under the project's Apache 2.0 License.