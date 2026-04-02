# YAPS-rs

**Yet Another Photo Sorter** — rewritten in Rust.

<!-- Badges -->
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
<!-- [![CI](https://github.com/example/yaps-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/example/yaps-rs/actions) -->
<!-- [![Crates.io](https://img.shields.io/crates/v/yaps.svg)](https://crates.io/crates/yaps) -->

Organize photos and videos into clean directory structures based on EXIF metadata. Pure Rust — no external binaries like `exiftool` required.

## Features

- **Pure Rust** — zero FFI, no C dependencies, no external tools
- **Fast hashing** — BLAKE3 for duplicate detection (significantly faster than MD5/SHA)
- **Parallel processing** — file scanning and EXIF extraction via [rayon](https://github.com/rayon-rs/rayon)
- **Flexible patterns** — 26 EXIF/file metadata tags for folder and filename templates
- **Multiple file operations** — copy, move, hardlink, or symlink
- **Smart conflict handling** — skip, rename with auto-increment, or overwrite
- **Duplicate detection** — persistent per-folder hash stores survive across runs
- **Graceful degradation** — files without EXIF data go to a configurable `[NoExifData]` folder
- **Dry-run mode** — preview all operations before committing
- **TOML configuration** — save and reuse sorting presets
- **CLI and GUI** — full-featured terminal interface and [iced](https://github.com/iced-rs/iced) graphical interface

## Installation

### From source

```sh
# CLI
cargo install --path crates/yaps-cli

# GUI
cargo install --path crates/yaps-gui
```

Requires **Rust 1.75+**.

## Quick Start

Sort photos by year and month, with day-stamped filenames:

```sh
yaps \
  --source ~/Photos/Unsorted \
  --target ~/Photos/Sorted \
  --folder-pattern "{year}/{month}" \
  --file-pattern "{day}-{month_short}-{hour}{minute}{second}-{filename}"
```

Preview first with `--dry-run`:

```sh
yaps -s ~/Photos/Unsorted -t ~/Photos/Sorted --dry-run
```

### What happens

```
~/Photos/Unsorted/IMG_1234.jpg  (taken 2024-03-15 14:30:05)
  → ~/Photos/Sorted/2024/03-March/15-Mar-143005-IMG_1234.jpg

~/Photos/Unsorted/DSC_5678.jpg  (no EXIF data)
  → ~/Photos/Sorted/[NoExifData]/DSC_5678.jpg
```

## Pattern System

Patterns use `{tag}` placeholders to build folder and file paths from metadata.

### Available Tags

| Category | Tags |
|----------|------|
| **Date & Time** | `{year}` `{month}` `{month_short}` `{month_long}` `{day}` `{day_short}` `{day_long}` `{hour}` `{minute}` `{second}` `{week}` |
| **Camera** | `{make}` `{model}` `{lens}` |
| **Exposure** | `{iso}` `{aperture}` `{shutter}` `{focal}` |
| **Dimensions** | `{width}` `{height}` `{orientation}` |
| **GPS** | `{gps_lat}` `{gps_lon}` |
| **File** | `{media_type}` `{filename}` `{ext}` |

### Tag Details

| Tag | Example Output | Description |
|-----|---------------|-------------|
| `{year}` | `2024` | Four-digit year |
| `{month}` | `03` | Zero-padded month number |
| `{month_short}` | `Mar` | Abbreviated month name |
| `{month_long}` | `March` | Full month name |
| `{day}` | `15` | Zero-padded day |
| `{day_short}` | `Fri` | Abbreviated weekday |
| `{day_long}` | `Friday` | Full weekday name |
| `{hour}` | `14` | 24-hour format |
| `{minute}` | `30` | Zero-padded minute |
| `{second}` | `05` | Zero-padded second |
| `{week}` | `11` | ISO week number |
| `{make}` | `Canon` | Camera manufacturer |
| `{model}` | `EOS R5` | Camera model |
| `{lens}` | `RF 24-70mm` | Lens description |
| `{iso}` | `400` | ISO sensitivity |
| `{aperture}` | `f/2.8` | F-number |
| `{shutter}` | `1/250` | Shutter speed |
| `{focal}` | `50mm` | Focal length |
| `{width}` | `6000` | Image width in pixels |
| `{height}` | `4000` | Image height in pixels |
| `{orientation}` | `landscape` | Orientation label |
| `{gps_lat}` | `37.7749` | GPS latitude (decimal) |
| `{gps_lon}` | `-122.4194` | GPS longitude (decimal) |
| `{media_type}` | `image` | `image`, `video`, or `audio` |
| `{filename}` | `IMG_1234` | Original filename (no extension) |
| `{ext}` | `jpg` | File extension (lowercase) |

Missing fields resolve to `unknown`. Escape literal braces with `{{` and `}}`.

### Pattern Examples

```sh
# By camera and date
--folder-pattern "{make}/{model}/{year}/{month_long}"

# By media type
--folder-pattern "{media_type}/{year}"

# Keep original name with timestamp prefix
--file-pattern "{year}{month}{day}_{hour}{minute}{second}_{filename}"

# By GPS coordinates (for geotagged photos)
--folder-pattern "{year}/{gps_lat},{gps_lon}"
```

## CLI Reference

```
yaps - Yet Another Photo Sorter

USAGE:
    yaps [OPTIONS] --source <PATH> --target <PATH>

OPTIONS:
    -s, --source <PATH>            Source directory containing photos to organize
    -t, --target <PATH>            Target directory for organized output
        --folder-pattern <PATTERN> Folder structure pattern [default: {year}/{month}]
        --file-pattern <PATTERN>   Filename pattern [default: {day}-{month_short}-{hour}{minute}{second}-{filename}]
        --operation <MODE>         File operation: copy, move, hardlink, symlink [default: copy]
        --on-conflict <STRATEGY>   Conflict resolution: skip, rename, overwrite [default: skip]
        --no-dedup                 Disable duplicate detection
        --keep-duplicates          Copy duplicates to [Duplicates] folder instead of skipping
        --no-recursive             Don't recurse into subdirectories
        --dry-run                  Preview operations without executing them
    -v, --verbose...               Increase verbosity (-v info, -vv debug, -vvv trace)
        --config <PATH>            Load configuration from a TOML file
    -h, --help                     Print help
    -V, --version                  Print version
```

## Configuration

Save sorting presets as TOML files:

```toml
source = "/home/user/Photos/Unsorted"
target = "/home/user/Photos/Sorted"
recursive = true
file_operation = "copy"
conflict_strategy = "rename"
detect_duplicates = true
duplicate_strategy = "skip"
folder_pattern = "{year}/{month}"
file_pattern = "{day}-{month_short}-{hour}{minute}{second}-{filename}"
dry_run = false
no_exif_folder = "[NoExifData]"
duplicates_folder = "[Duplicates]"
```

```sh
yaps --config my-preset.toml
```

CLI flags override config file values.

## Supported Formats

| Type | Extensions |
|------|-----------|
| **Images** | jpg, jpeg, png, gif, bmp, tiff, tif, webp, heic, heif, avif, raw, cr2, cr3, nef, arw, orf, rw2, dng, raf, pef, srw |
| **Videos** | mp4, mov, avi, mkv, wmv, flv, m4v, 3gp, mts, m2ts |

## Architecture

```
yaps-rs/
├── crates/
│   ├── yaps-core/     # Library — all sorting logic, zero UI dependencies
│   │   └── src/
│   │       ├── config.rs      # Config, enums (FileOperation, ConflictStrategy, etc.)
│   │       ├── error.rs       # Typed errors via thiserror
│   │       ├── exif/          # EXIF reader, metadata types, date parsing
│   │       ├── hash/          # BLAKE3 hasher, persistent hash store
│   │       ├── ops/           # Scanner, file ops, conflict resolver, organizer
│   │       ├── pattern/       # Template parser, formatter, tag definitions
│   │       └── report.rs      # Run statistics
│   ├── yaps-cli/      # CLI binary (clap + tracing)
│   └── yaps-gui/      # GUI binary (iced + rfd file dialogs)
├── tests/             # Integration tests
└── benches/           # Benchmarks
```

**Design principles:**

- **UI-agnostic core** — `yaps-core` has no UI dependencies; both CLI and GUI consume it
- **Streaming I/O** — BLAKE3 hashing uses 8 KB buffers for constant memory usage
- **Parallel by default** — rayon for scanning and EXIF extraction
- **Persistent state** — hash stores are saved per-folder, surviving across runs
- **No unsafe** — `unsafe_code = "forbid"` enforced workspace-wide

## Performance

Compared to the original C++ YAPS implementation:

| Aspect | C++ (original) | Rust (yaps-rs) |
|--------|----------------|----------------|
| EXIF parsing | exiftool (subprocess) | kamadak-exif (in-process) |
| Hashing | MD5 | BLAKE3 (SIMD-accelerated) |
| Parallelism | Manual threading | rayon work-stealing |
| Binary size | Depends on linked libs | Single static binary |
| Dependencies | exiftool required | None (pure Rust) |

The release profile enables LTO and single codegen unit for maximum optimization.

## Building from Source

```sh
git clone https://github.com/example/yaps-rs.git
cd yaps-rs

# Debug build (all crates)
cargo build

# Optimized release build (all crates)
cargo build --release

# Build individual binaries
cargo build --release -p yaps-cli   # CLI only
cargo build --release -p yaps-gui   # GUI only

# Binaries at:
#   target/release/yaps       (CLI)
#   target/release/yaps-gui   (GUI)
```

## Testing

```sh
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Lint
cargo clippy -- -D warnings

# Format check
cargo fmt --check
```

## Benchmarks

```sh
# Run all benchmarks
cargo bench -p yaps-core

# Run specific benchmark suite
cargo bench -p yaps-core --bench hashing_bench
cargo bench -p yaps-core --bench scanning_bench
```

## License

[MIT](LICENSE)
