# Dx (Developer Experience) - AI Coding Assistant Instructions

## Project Overview
Dx is a comprehensive developer experience ecosystem written in Rust, consisting of multiple specialized tools including dx-style (CSS utility generator), dx-check (linting/formatting), dx-i18n (internationalization), dx-media (asset processing), and others. The system uses FlatBuffers for high-performance binary serialization of TOML configuration files, enabling lightning-fast operations.

Dx tools work primarily in the background through intelligent file watching - no manual commands needed. The system automatically detects file changes and performs the appropriate actions (formatting, linting, style generation) through a process called "forging." This saves developers time by eliminating repetitive build commands and manual tooling orchestration.

This specific repository focuses on **dx-style**, currently the most powerful tool in the Dx ecosystem. Dx-style is a high-performance CSS utility generator similar to Tailwind CSS that parses HTML files to extract CSS class names and generates corresponding utility styles on-demand using precompiled binary format for optimal performance.

## Current Status & Roadmap
- ✅ **Basic dx-style logic**: Complete - HTML parsing, CSS generation, file watching
- 🚧 **Advanced dx-style features**: In development - animations, theming, synchronization
- 📋 **Future tools**: dx-check, dx-i18n, dx-media (mentioned for context, implemented separately)

## Architecture Overview

### Core Components
- **Style Engine** (`src/core/engine/`): Loads precompiled styles from `.dx/style/style.bin`
- **Parser** (`src/parser/`): Extracts CSS classes from HTML using fast byte-level parsing
- **Generator** (`src/generator/`): Creates CSS rules from extracted classes
- **Watcher** (`src/watcher/`): Monitors file changes with debounced rebuilds
- **Core** (`src/core/`): Orchestrates style rebuilding and state management

### Data Flow
1. HTML files are parsed to extract class names
2. Classes are matched against the style engine's precompiled rules
3. CSS utilities are generated and written to output files
4. File watcher triggers rebuilds on changes

### Forging System
Dx implements "forging" - intelligent background processing that automatically:
- Detects file changes across the project
- Determines appropriate actions (format, lint, generate styles)
- Executes operations without manual intervention
- Maintains performance through debouncing and incremental processing

## Configuration System

### Main Config (`.dx/config.toml`)
```toml
[paths]
html_dir = "playgrounds"
index_file = "playgrounds/index.html"
css_file = "playgrounds/style.css"

[watch]
debounce_ms = 1
```

### Style Definitions (`.dx/style/*.toml`)
- **`static.toml`**: One-to-one class-to-CSS mappings (e.g., `flex = "display: flex;"`)
- **`dynamic.toml`**: Enumerated values with variants (e.g., `bg-red-500`, `text-center`)
- **`generators.toml`**: Numeric scales with multipliers/units (e.g., `w-50` → `width: 12.5rem`)
- **`colors.toml`**: Color definitions with OKLCH color space
- **`screens.toml`**: Responsive breakpoint definitions
- **`states.toml`**: Pseudo-class variants (hover, focus, etc.)

## Key Development Patterns

### Style Definition Formats

**Static Classes:**
```toml
[static]
flex = "display: flex;"
items-center = "align-items: center;"
```

**Dynamic Classes:**
```toml
[dynamic]
"bg-color|background-color" = { red = "red", blue = "blue" }
"display|display" = { block = "block", flex = "flex" }
```

**Generated Classes:**
```toml
[generators]
"w|width" = { multiplier = 0.25, unit = "rem" }
"h|height" = { multiplier = 0.25, unit = "rem" }
```

### Build Process
- Style definitions are compiled to FlatBuffers binary during `cargo build`
- Binary is loaded at runtime by the style engine
- CSS generation happens on-demand based on HTML class usage
- TOML configuration files are transformed into high-performance binary format for optimal speed

### File Watching & Rebuilding
- Debounced file watching (default 250ms)
- Automatic CSS regeneration when HTML changes
- Environment variables control rebuild behavior:
  - `DX_FORCE_FULL=1`: Force complete rebuild
  - `DX_FORCE_FORMAT=1`: Trigger formatting pass
  - `DX_SILENT_FORMAT=1`: Suppress format logging

## Development Workflow

### Adding New Utilities
1. Define styles in appropriate `.dx/style/*.toml` file
2. Run `cargo build` to regenerate `style.bin`
3. Use classes in HTML - they'll be auto-generated in CSS output

### Testing Changes
- Modify HTML files in `playgrounds/` directory
- Changes trigger automatic CSS regeneration
- Check generated `playgrounds/style.css` for output

### Performance Considerations
- Uses memory-mapped files for style binary loading
- Fast HTML parsing with `memchr` for byte-level operations
- Parallel processing with `rayon` for large files
- Incremental rebuilds avoid regenerating unchanged utilities

## Common Tasks

### Adding Color Variants
```toml
# In colors.toml
[colors]
blue-500 = "oklch(63.7% .237 25.331)"
```

### Creating Responsive Utilities
```toml
# In screens.toml
[screens]
md = "@media (min-width: 768px)"
lg = "@media (min-width: 1024px)"
```

### Adding Pseudo-class Support
```toml
# In states.toml
[states]
hover = ":hover"
focus = ":focus"
```

## Current Development Focus

### Advanced dx-style Features (In Development)
- **Animations**: Keyframe-based animation utilities
- **Theming**: Dynamic theme switching and custom property management
- **Synchronization**: Cross-file dependency tracking and coordinated rebuilds
- **Additional features**: Extended responsive utilities, advanced selectors, and performance optimizations

## Code Style & Conventions

### Rust Patterns
- Extensive use of `AHashMap` and `AHashSet` for performance
- Memory-mapped file I/O with `memmap2`
- FlatBuffers for binary serialization
- Rayon for parallel processing
- Error handling with `Box<dyn std::error::Error>`

### CSS Generation
- CSS custom properties for theming (`--color-*`, `--bg-opacity`)
- Layered architecture: `@layer theme, base, components, utilities`
- OKLCH color space for better color manipulation
- Automatic CSS escaping with `cssparser`

### File Organization
- TOML configuration files in `.dx/` directory
- Generated binaries alongside configs
- HTML playgrounds in `playgrounds/` directory
- Source code follows standard Rust module structure

## Debugging & Troubleshooting

### Common Issues
- **Styles not generating**: Check if classes are properly defined in TOML files
- **Watch not triggering**: Verify file paths in `.dx/config.toml`
- **Performance issues**: Check `DX_MMAP_THRESHOLD` environment variable

### Debug Environment Variables
- `DX_DEBUG=1`: Enable debug logging
- `DX_WATCH_POLL_MS=100`: Use polling instead of native file watching
- `DX_WATCH_RAW=1`: Bypass debouncer for immediate rebuilds

## Integration Points

### External Dependencies
- **FlatBuffers**: Binary serialization format for high-performance TOML compilation
- **LightningCSS**: CSS parsing and processing
- **Notify**: Cross-platform file watching
- **Tokio**: Async runtime for I/O operations

### Build Integration
- Custom build script (`build.rs`) compiles TOML to FlatBuffers
- Cargo features for optional dependencies (`image`, `std`)
- Development vs production build configurations

### CLI vs Background Operation
- Dx provides CLI commands for manual operations
- Primary operation mode is background "forging" - automatic, command-free processing
- File watching enables real-time responses without developer intervention

Remember: Dx generates CSS utilities on-demand based on HTML usage. Define your styles in TOML, build the binary, then use classes in HTML - the system handles the rest automatically through intelligent background forging.