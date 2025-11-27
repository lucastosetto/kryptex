# Contributing to Perptrix

Thanks for your interest in improving Perptrix! Please follow these guidelines
to keep the workflow smooth.

## Getting Started

1. **Discuss first**: Open an issue describing the bug or feature you plan to
   tackle. This helps avoid duplicated work and align on scope.
2. **Fork and branch**: Fork the repo and create a descriptive branch, e.g.
   `git checkout -b feature/hyperliquid-adapter`.
3. **Keep changes focused**: Prefer smaller PRs that address a single problem.

## Development Workflow

- Ensure you have a recent Rust toolchain (Rust 1.70+).
- Run the following before each pull request:
  - `cargo fmt`
  - `cargo clippy -- -D warnings`
  - `cargo test`
- Add or update tests when you introduce behavior changes.
- Update documentation/README when you add new features or config options.

## Pull Requests

When opening a PR:

1. Reference the related issue.
2. Describe the motivation, approach, and any trade-offs.
3. Include testing details (commands run, scenarios covered).
4. Be ready to iterate on review feedback.

## Licensing

By contributing to Perptrix, you agree that your contributions will be licensed
under the MIT License, the same license that covers the rest of the project.

