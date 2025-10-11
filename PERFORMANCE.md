# Performance Optimizations in dx-style

This document describes the performance optimizations implemented in dx-style and provides guidance on how to further optimize the build tool for maximum performance.

## Implemented Optimizations

### 1. Fast Hashing with AHash

**What it is:** dx-style uses `ahash` (AHash) instead of Rust's default `HashMap` and `HashSet`, which use the cryptographically-secure SipHash algorithm.

**Why it matters:** AHash is a non-cryptographic hash function that's significantly faster for general-purpose use. Since build tools process trusted input, we don't need the collision-resistance guarantees of cryptographic hashing.

**Performance impact:** ~20-30% faster hash operations in hot paths like class name lookups and caching.

**Implementation:**

- All `HashMap<K, V>` → `AHashMap<K, V>`
- All `HashSet<T>` → `AHashSet<T>`
- Used throughout: parser, engine, generators, and core modules

**Alternative:** The `rustc-hash` crate (FxHash) is also available if you want to experiment with different hashing strategies. It's what the Rust compiler uses internally.

```rust
// To try FxHashMap instead:
use rustc_hash::FxHashMap;
let map: FxHashMap<String, String> = FxHashMap::default();
```

### 2. Parallel CSS Generation

**What it is:** CSS rule generation uses `rayon` parallel iterators to generate CSS for multiple utility classes simultaneously across CPU cores.

**Why it matters:** Modern machines have multiple cores. Generating CSS for hundreds or thousands of utility classes is an embarrassingly parallel problem - each class-to-CSS conversion is independent.

**Performance impact:** ~2-4x speedup on multi-core machines when processing >100 classes. Automatic fallback to sequential processing for small class sets to avoid parallelization overhead.

**Implementation:**

- Threshold: 100+ classes trigger parallel processing
- Each thread generates CSS chunks independently
- Results are combined sequentially to maintain order
- Located in `src/generator/mod.rs`

**Tuning:**

```rust
// Adjust the parallelization threshold in generator/mod.rs:
if collected.len() > 100 {  // Change this number based on profiling
    // Parallel path...
}
```

### 3. Link Time Optimization (LTO)

**What it is:** LTO allows the compiler to perform optimizations across the entire codebase at once, rather than per-crate.

**Why it matters:** Enables more aggressive inlining, dead code elimination, and whole-program analysis.

**Performance impact:** 5-15% faster execution, smaller binary size.

**Configuration in `Cargo.toml`:**

```toml
[profile.release]
lto = "fat"           # Full LTO across all crates
codegen-units = 1     # Single codegen unit for maximum optimization
```

**Trade-offs:**

- **Pros:** Faster runtime, smaller binaries
- **Cons:** Much slower compile times (2-5x longer)
- **Recommendation:** Use for release builds only

### 4. Optimized Release Profile

The release profile is configured for maximum runtime performance:

```toml
[profile.release]
opt-level = 3         # Maximum optimization level
lto = "fat"           # Full link-time optimization
codegen-units = 1     # Single codegen unit (better optimization, slower compilation)
strip = true          # Remove debug symbols for smaller binaries
panic = "abort"       # Smaller binary, faster panic handling
```

### 5. String Interning (Optional Feature)

**What it is:** String interning stores a single copy of each unique string and uses cheap integer keys for comparisons.

**Why it matters:** If you're processing the same strings repeatedly (class names, CSS properties like "background-color"), interning can reduce memory usage and speed up string comparisons.

**Status:** Available via the `lasso` crate but not yet integrated. Enable with feature flag:

```bash
cargo build --release --features string-interning
```

**Integration example:**

```rust
use lasso::{Rodeo, Spur};

let mut interner = Rodeo::new();
let bg_key: Spur = interner.get_or_intern("background-color");
let text_key: Spur = interner.get_or_intern("text-align");

// Comparing keys is much faster than comparing full strings
if bg_key == text_key { /* ... */ }

// Get the string back when needed
let property = interner.resolve(&bg_key);
```

## Advanced Optimization: Profile-Guided Optimization (PGO)

PGO is one of the most powerful optimization techniques available, providing 5-15% additional performance improvements on top of standard optimizations.

### What is PGO?

Profile-Guided Optimization is a three-step process:

1. **Build with instrumentation** - Create a special build that records which code paths are executed
2. **Run with representative workload** - Execute the program on realistic data to generate profile information
3. **Rebuild with profile data** - Recompile using the profile to optimize hot paths

### When to Use PGO

✅ **Use PGO when:**

- You're building for production/distribution
- You have representative benchmark workloads
- You want maximum performance
- Compilation time isn't critical

❌ **Skip PGO when:**

- Doing local development
- Rapid iteration is needed
- You don't have good benchmark data

### PGO Setup Instructions

#### Step 1: Build with Instrumentation

```bash
# Set the instrumentation flag
export RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data"

# Build the instrumented binary
cargo build --release --target=x86_64-pc-windows-msvc
```

On Windows PowerShell:

```powershell
$env:RUSTFLAGS="-Cprofile-generate=C:\temp\pgo-data"
cargo build --release
```

#### Step 2: Run Representative Workloads

Run your program with workloads that represent real-world usage. For dx-style, this means:

```bash
# Create a test project with many HTML files
mkdir -p test-workload/pages
for i in {1..100}; do
  echo "<div class='flex items-center justify-between p-4 bg-blue-500 text-white hover:bg-blue-600'>" > test-workload/pages/page$i.html
  echo "  <span class='text-lg font-bold'>Page $i</span>" >> test-workload/pages/page$i.html
  echo "  <button class='px-4 py-2 rounded-lg shadow-md'>Click me</button>" >> test-workload/pages/page$i.html
  echo "</div>" >> test-workload/pages/page$i.html
done

# Run the instrumented binary on this workload
cd test-workload
../target/release/style build

# Run multiple times with different scenarios
# - Large files
# - Many small files
# - Different class combinations
# The more realistic the workload, the better the profile
```

On Windows:

```powershell
# Create test files
for ($i=1; $i -le 100; $i++) {
    @"
<div class='flex items-center justify-between p-4 bg-blue-500 text-white hover:bg-blue-600'>
  <span class='text-lg font-bold'>Page $i</span>
  <button class='px-4 py-2 rounded-lg shadow-md'>Click me</button>
</div>
"@ | Out-File -FilePath "test-workload\pages\page$i.html"
}

# Run the tool
.\target\release\style.exe build
```

#### Step 3: Build with Profile Data

```bash
# Merge profile data (if using LLVM tools)
llvm-profdata merge -o /tmp/pgo-data/merged.profdata /tmp/pgo-data/*.profraw

# Build the optimized binary
export RUSTFLAGS="-Cprofile-use=/tmp/pgo-data/merged.profdata -Cllvm-args=-pgo-warn-missing-function"
cargo build --release
```

On Windows:

```powershell
# Merge profile data
llvm-profdata merge -o C:\temp\pgo-data\merged.profdata C:\temp\pgo-data\*.profraw

# Build optimized
$env:RUSTFLAGS="-Cprofile-use=C:\temp\pgo-data\merged.profdata"
cargo build --release
```

### Troubleshooting PGO

**Problem:** `llvm-profdata` command not found

- **Solution:** Install LLVM tools: `rustup component add llvm-tools-preview`
- Find the tool: `$(rustc --print sysroot)/lib/rustlib/x86_64-pc-windows-msvc/bin/llvm-profdata`

**Problem:** Profile data warnings

- **Solution:** This is normal if your workload doesn't exercise all code paths. Use `-Cllvm-args=-pgo-warn-missing-function` to suppress warnings.

**Problem:** Significantly slower with PGO

- **Solution:** Your profile data may not be representative. Re-run with more realistic workloads.

### PGO Build Script Example

Create `scripts/build-pgo.sh`:

```bash
#!/bin/bash
set -e

PGO_DATA=/tmp/pgo-data
WORKLOAD_DIR=./pgo-workload

echo "==> Step 1: Building instrumented binary..."
rm -rf $PGO_DATA
mkdir -p $PGO_DATA
RUSTFLAGS="-Cprofile-generate=$PGO_DATA" cargo build --release

echo "==> Step 2: Running workloads to generate profile data..."
cd $WORKLOAD_DIR
../target/release/style build
../target/release/style watch &
WATCH_PID=$!
sleep 5
kill $WATCH_PID
cd ..

echo "==> Step 3: Merging profile data..."
LLVM_PROFDATA=$(rustc --print sysroot)/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-profdata
$LLVM_PROFDATA merge -o $PGO_DATA/merged.profdata $PGO_DATA

echo "==> Step 4: Building optimized binary with PGO..."
RUSTFLAGS="-Cprofile-use=$PGO_DATA/merged.profdata" cargo build --release

echo "==> Done! PGO-optimized binary is in target/release/"
```

Make it executable and run:

```bash
chmod +x scripts/build-pgo.sh
./scripts/build-pgo.sh
```

## Performance Testing

### Benchmarking

dx-style includes benchmark support via `criterion`:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench parser_bench

# Compare with baseline
cargo bench -- --save-baseline main
# ... make changes ...
cargo bench -- --baseline main
```

### Profiling

#### CPU Profiling with `perf` (Linux)

```bash
# Build with debug symbols
cargo build --release

# Profile the application
perf record --call-graph dwarf target/release/style build

# Analyze the results
perf report
```

#### CPU Profiling with `cargo-flamegraph`

```bash
# Install
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin style -- build

# Open flamegraph.svg in a browser
```

#### Memory Profiling with `valgrind` (Linux)

```bash
# Install massif
sudo apt-get install valgrind

# Profile memory usage
valgrind --tool=massif --massif-out-file=massif.out target/release/style build

# Visualize
ms_print massif.out
```

#### Windows Performance Analysis

Use Windows Performance Analyzer (WPA):

```powershell
# Start recording
wpr -start CPU

# Run your program
.\target\release\style.exe build

# Stop recording
wpr -stop output.etl

# Analyze in WPA
wpa output.etl
```

## Performance Best Practices

### 1. Use Release Builds for Benchmarking

```bash
# Always benchmark release builds
cargo build --release
cargo bench
```

### 2. Disable CPU Frequency Scaling (Linux)

```bash
# Set CPU to performance mode
sudo cpupower frequency-set --governor performance

# Run benchmarks
cargo bench

# Restore powersave mode
sudo cpupower frequency-set --governor powersave
```

### 3. Close Background Applications

Close browsers, IDEs, and other resource-intensive applications when benchmarking.

### 4. Run Multiple Iterations

```bash
# Run benchmarks multiple times
for i in {1..5}; do cargo bench; done
```

### 5. Profile Before Optimizing

> "Premature optimization is the root of all evil" - Donald Knuth

Always profile to find actual bottlenecks before optimizing. Use tools like `perf`, `flamegraph`, or `criterion` to identify hot paths.

## Future Optimization Opportunities

### 1. SIMD (Single Instruction Multiple Data)

Use SIMD instructions for byte-level operations in the HTML parser:

```rust
// Potential SIMD optimization for class extraction
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// Process 16 bytes at once looking for whitespace
```

### 2. Memory Pool Allocation

Implement a memory pool for frequently allocated small objects:

```rust
use bumpalo::Bump;

let arena = Bump::new();
let classes = arena.alloc_slice_copy(&extracted_classes);
```

### 3. Incremental Compilation

Cache intermediate results between rebuilds:

```rust
// Store hash of HTML content -> generated CSS
// Skip regeneration if content hasn't changed
```

### 4. Lazy Evaluation

Only generate CSS for classes that are actually used:

```rust
// Parse HTML first, then generate only needed CSS
// Instead of pre-generating all utilities
```

## Comparing Performance

### Before and After Optimization

Run these benchmarks to compare:

```bash
# Checkout the commit before optimizations
git checkout <before-commit>
cargo build --release
hyperfine './target/release/style build'

# Checkout the commit after optimizations
git checkout <after-commit>
cargo build --release
hyperfine './target/release/style build'
```

### Expected Performance Gains

| Optimization | Expected Speedup | Compile Time Impact |
|-------------|------------------|---------------------|
| AHash | 20-30% (hash operations) | None |
| Parallel CSS Gen | 2-4x (on 4+ cores) | None |
| LTO | 5-15% | 2-5x slower |
| PGO | 5-15% | 2-3x slower |
| **Combined** | **50-100%** | **4-8x slower** |

**Note:** Actual gains depend on workload, hardware, and project size. Always measure!

## Questions and Feedback

If you discover additional optimization opportunities or have questions about these techniques, please:

1. Profile the application to identify bottlenecks
2. Create a benchmark demonstrating the issue
3. Open an issue with profiling data and proposed solution
4. Submit a PR with the optimization

Happy optimizing! 🚀
