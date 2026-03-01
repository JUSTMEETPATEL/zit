# Contributing to Zit

Thank you for your interest in contributing to Zit! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and considerate in all interactions. We welcome contributors of all experience levels.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs) (latest stable)
- Git 2.30+
- A modern terminal with TrueColor support

### Development Setup

1. Fork and clone the repository:
   ```bash
   git clone https://github.com/YOUR_USERNAME/zit.git
   cd zit
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test --all-targets
   ```

4. Run the application:
   ```bash
   cargo run
   ```

## Development Workflow

### Before Submitting

1. **Format your code**:
   ```bash
   cargo fmt
   ```

2. **Run clippy**:
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```

3. **Run all tests**:
   ```bash
   make check
   ```

### Commit Messages

Follow conventional commit format:
- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `refactor:` Code refactoring
- `test:` Test additions or changes
- `chore:` Maintenance tasks

Example: `feat: add branch rename confirmation dialog`

### Pull Request Process

1. Create a feature branch from `main`
2. Make your changes with clear, focused commits
3. Ensure all tests pass
4. Update documentation if needed
5. Submit a pull request with a clear description

## Project Structure

```
src/
├── main.rs          # Entry point
├── app.rs           # App state and view routing
├── config.rs        # Configuration management
├── event.rs         # Event handling
├── ai/              # AI client integration
├── git/             # Git command execution
└── ui/              # TUI views and components

aws/
├── lambda/          # Lambda function code
└── infrastructure/  # SAM/CloudFormation templates

tests/
└── integration.rs   # Integration tests
```

## Testing

### Unit Tests

```bash
cargo test
```

### Lambda Tests

```bash
cd aws
python3 -m pytest tests/ -v
```

## Reporting Issues

When reporting bugs, please include:
- Zit version (`zit --version`)
- Operating system and terminal emulator
- Git version (`git --version`)
- Steps to reproduce
- Expected vs actual behavior

## Feature Requests

We welcome feature suggestions! Please open an issue describing:
- The problem you're trying to solve
- Your proposed solution
- Any alternatives you've considered

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
