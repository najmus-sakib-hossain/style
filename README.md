# Dx Style

**Enhance Developer Experience with Lightning-Fast CSS Generation**

A high-performance CSS utility generator written in Rust, featuring on-demand style generation, intelligent file watching, and advanced optimizations for maximum speed.

## 🚀 Performance

dx-style is engineered for extreme performance:

- ⚡ **Sub-20µs** class additions/removals
- 🔥 **SIMD-accelerated** HTML parsing
- 🧵 **Parallel CSS generation** for large files
- 💾 **Memory-efficient** with string interning and arena allocation
- 🎯 **Profile-Guided Optimization** for 10-20% extra speed

### Advanced Optimizations

This project includes cutting-edge performance optimizations:

1. **String Interning** - Reduces memory by 20-30% through deduplication
2. **FxHash** - 10-20% faster hashing in hot paths
3. **Arena Allocation** - Zero-overhead batch CSS generation
4. **PGO** - Profile-guided optimization for real-world workloads

See [ADVANCED_OPTIMIZATIONS.md](.github/ADVANCED_OPTIMIZATIONS.md) for details.

## 📦 Building

### Standard Build

```bash
cargo build --release
```

### Maximum Performance Build (with PGO)

```bash
# Linux/macOS
./scripts/build_pgo.sh

# Windows
scripts\build_pgo.bat
```

This builds with Profile-Guided Optimization for 30-50% better performance.

## 🎯 Features

- ✅ **FlatBuffers-based** precompiled style engine
- ✅ **Automatic background processing** ("forging")
- ✅ **Grouping syntax** for reusable class combinations
- ✅ **OKLCH color space** support
- ✅ **Memory-mapped I/O** for large CSS files
- ✅ **Incremental rebuilds** for instant updates

## 📊 Benchmarks

```bash
# Run full benchmark suite
cargo bench

# Quick performance test
cargo test --test performance_integration --release
```

Expected performance (with all optimizations):
- Add single class: ~10-12µs
- Parse 100 classes: ~320µs  
- Parse 1000 classes: ~1.9ms
- Full rebuild: ~5.5ms

## 🔧 Configuration

All advanced optimizations are enabled by default. To customize:

```toml
[features]
default = ["std", "image", "string-interning", "fast-hash", "arena-alloc"]
```

## 📚 Documentation

- [Performance Guide](.github/PERFORMANCE.md)
- [Advanced Optimizations](.github/ADVANCED_OPTIMIZATIONS.md)
- [Optimization Summary](.github/OPTIMIZATION_SUMMARY.md)

## 🤝 Contributing

Contributions welcome! This project uses advanced Rust patterns and performance techniques. See the optimization guides for implementation details.

## 📄 License

MIT License - See LICENSE file for details
