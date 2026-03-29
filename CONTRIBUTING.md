# Contributing to Vanaspati

## Quick Start

```bash
git clone https://github.com/MacCracken/vanaspati.git
cd vanaspati
make check
```

## Workflow

1. Fork and create a feature branch from `main`
2. Write code + tests
3. Run `make check` (must pass: fmt, clippy, test, audit)
4. Run `make bench` if touching hot paths
5. Update CHANGELOG.md
6. Open a PR against `main`

## Code Standards

- `cargo fmt` — no exceptions
- `cargo clippy --all-features --all-targets -- -D warnings` — zero warnings
- `#[non_exhaustive]` on all public enums
- `#[must_use]` on all pure functions
- No `unwrap()` or `panic!()` in library code
- Units in comments for all physical quantities
- Minimum 80% test coverage target

## Running Checks

```bash
make check      # fmt + clippy + test + audit
make test       # cargo test --all-features
make bench      # benchmarks with CSV history
make coverage   # LLVM coverage report (HTML)
make doc        # docs with warnings = errors
```

## Commit Messages

- Use imperative mood: "add growth model" not "added growth model"
- Keep the first line under 72 characters
- Reference issues when applicable

## License

By contributing, you agree that your contributions will be licensed under the GPL-3.0 license.
