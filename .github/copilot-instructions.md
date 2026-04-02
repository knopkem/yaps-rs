# Copilot Instructions for yaps-rs

## Project Overview
yaps-rs is a high-performance photo sorting tool written in pure Rust. It organizes photos
into user-defined directory structures based on EXIF metadata, with BLAKE3-based duplicate
detection. The project is structured as a Cargo workspace with three crates:
- `yaps-core`: Core library with all business logic (no UI dependencies)
- `yaps-cli`: Command-line interface binary
- `yaps-gui`: GUI binary using the iced framework

## Rust Coding Standards

### General
- Target Rust edition 2021, minimum MSRV 1.75
- Run `cargo clippy -- -D warnings` and `cargo fmt --check` before committing
- Prefer `&str` / `&Path` over `String` / `PathBuf` in function parameters when ownership is not needed
- Use `impl AsRef<Path>` for functions accepting paths
- Avoid `.unwrap()` and `.expect()` in library code — always propagate errors with `?`
- `.unwrap()` is acceptable only in tests and when the invariant is provably guaranteed
- Prefer iterators and combinators over manual loops where it improves clarity
- Use `#[must_use]` on functions that return values that should not be ignored

### Error Handling
- `yaps-core` uses `thiserror` for typed error enums — each module may define its own
- Binary crates (`yaps-cli`, `yaps-gui`) use `anyhow` for top-level error handling
- Always add `.context()` when propagating errors across module boundaries
- Never silently swallow errors — at minimum log them with `tracing::warn!`

### Module Structure
- Each module has a `mod.rs` that re-exports the public API
- Keep internal implementation details private; expose only what's needed
- Use `pub(crate)` for items shared within yaps-core but not with external consumers
- Group related types in a single file; split when a file exceeds ~300 lines

### Naming Conventions
- Types: `PascalCase` (e.g., `ExifMetadata`, `PatternTag`, `HashStore`)
- Functions/methods: `snake_case` (e.g., `read_exif`, `format_pattern`)
- Constants: `SCREAMING_SNAKE_CASE`
- Enum variants: `PascalCase`
- File names: `snake_case.rs`

### Testing
- **Every public function must have at least one unit test**
- Place unit tests in `#[cfg(test)] mod tests` at the bottom of each file
- Place integration tests in the workspace `tests/` directory
- Use `tempfile` crate for any test that touches the file system
- Test both success and error paths
- Use descriptive test names: `test_parse_pattern_with_unknown_tag_returns_error`
- Aim for >80% code coverage on `yaps-core`

### Performance
- Use `rayon` for parallel file processing (scanning, hashing, EXIF extraction)
- Use streaming/buffered I/O for hashing — never read entire files into memory
- Prefer `&[u8]` slices over `Vec<u8>` allocations where possible
- Use `std::io::BufReader` with appropriate buffer sizes (8KB default)
- Benchmark critical paths with `criterion`

### Documentation
- All public types, traits, and functions must have `///` doc comments
- Include `# Examples` sections for key APIs
- Use `# Errors` section to document when functions return `Err`
- Module-level `//!` documentation for each module explaining its purpose

### Dependencies
- Pure Rust only — no FFI bindings or C library dependencies
- Prefer well-maintained crates with >1M downloads where possible
- Pin major versions in workspace Cargo.toml

### Git Practices
- Write conventional commit messages (e.g., `feat:`, `fix:`, `test:`, `refactor:`)
- Keep commits atomic — one logical change per commit
