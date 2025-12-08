# RustyWind xtask

Automation tasks for RustyWind development, written in Rust.

## Overview

This crate provides developer automation tools for the RustyWind project, replacing the previous Python scripts with native Rust implementations. All tasks can be run using `cargo xtask <command>`.

## Available Commands

### fuzz setup

Set up the fuzz test environment by building the RustyWind release binary and installing npm dependencies.

```bash
cargo xtask fuzz setup
```

**What it does:**
- Builds RustyWind with `cargo build --release`
- Installs npm dependencies with `npm install` in `tests/fuzz`
- Verifies that all prerequisites are in place

**Note:** The `fuzz run` command automatically runs setup if needed, so you typically don't need to run this manually. However, it's useful if you want to prepare the environment before running tests.

### fuzz run

Run fuzz tests in parallel with automatic failure analysis.

```bash
# run 25 rounds (default) with auto-detected workers
cargo xtask fuzz run

# run 100 rounds
cargo xtask fuzz run 100

# run 50 rounds with 4 workers
cargo xtask fuzz run 50 --workers 4
```

**What it does:**
- Automatically runs setup if needed (builds binary + installs npm deps)
- Pre-flight checks (RustyWind binary, npm, node_modules)
- Runs fuzz tests in parallel with progress bar
- Tracks pass/fail counts
- Generates aggregate statistics:
  - Total tests, passed, failed counts
  - Overall pass rate
  - Min/max/avg pass rates
  - Distribution histogram
- **Automatically analyzes failures:**
  - Categorizes CSS classes (custom, arbitrary, opacity, shadow, ring, border, color, filter, etc.)
  - Identifies top category mismatches
  - Shows specific class pairs (appearing 3+ times)
  - Saves detailed results to `tests/fuzz/tools/output/failure_analysis.txt`
- Reports failed rounds by error type

**Configuration:**
- Number of parallel workers auto-detected (CPU count, min 2, max 8)
- Override with `--workers` flag or `FUZZ_WORKERS` environment variable

**Replaces:** `test_many_rounds.py`, `collect_failures.py`, and `analyze_failures.py`

## Prerequisites

Commands require:
- Node.js and npm installed (system-wide)

The `fuzz run` command automatically ensures that:
- RustyWind binary is built (`cargo build --release`)
- npm dependencies are installed (`cd tests/fuzz && npm install`)

You can also run `cargo xtask fuzz setup` manually to prepare the environment.

## Implementation Details

### Architecture

```
xtask/
├── src/
│   ├── main.rs              # CLI entry point with clap
│   ├── commands/            # Command implementations
│   │   ├── setup.rs         # setup command
│   │   └── run.rs           # run command with integrated analysis
│   └── utils/               # Shared utilities
│       ├── categories.rs    # CSS class categorization
│       └── parser.rs        # Test output parsing
└── Cargo.toml
```

### Key Dependencies

- **clap** - CLI argument parsing
- **color-eyre** - Error handling
- **regex** - Test output parsing
- **rayon** - Parallel execution
- **indicatif** - Progress bars
- **which** - Binary detection
- **num_cpus** - CPU count detection

### Advantages Over Python Scripts

1. **Type Safety** - Catch errors at compile time
2. **Performance** - Faster parallel execution with rayon
3. **Integration** - Can import rustywind-core directly
4. **Consistency** - One language for entire project
5. **Tooling** - Better IDE support, debugging, profiling
6. **Dependencies** - No external runtime (Python) needed
7. **Cross-platform** - Rust handles platform differences
8. **Unified Workflow** - Run tests and analyze in one command

## Development

### Building

```bash
cargo build --package xtask
```

### Testing

```bash
cargo test --package xtask
```

### Adding New Commands

1. Create new file in `src/commands/`
2. Implement `pub fn run(...) -> Result<()>`
3. Add module to `src/commands/mod.rs`
4. Add variant to appropriate enum in `src/main.rs`
5. Add match arm in the match expression

### Code Style

- Start inline comments with lowercase
- Capitalize doc comments (///)
- Use `color-eyre` for error handling
- Minimize nesting in functions
- Use meaningful variable names
