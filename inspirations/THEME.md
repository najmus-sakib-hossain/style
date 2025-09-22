# Theme Generation from Color or Image

```rust
use colored::Colorize;
use style::core::color::{color::Argb, theme::ThemeBuilder};

fn color_to_theme(source: Argb) -> style::core::color::theme::Theme {
    ThemeBuilder::with_source(source).build()
}

fn local_image_to_theme<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<style::core::color::theme::Theme, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("failed to read file: {}", e))?;

    let mut img = style::core::color::image::ImageReader::read(bytes)
        .map_err(|e| format!("failed to decode image: {}", e))?;

    use style::core::color::image::FilterType; 
    img.resize(128, 128, FilterType::Lanczos3);

    let source = style::core::color::image::ImageReader::extract_color(&img);

    Ok(ThemeBuilder::with_source(source).build())
}

async fn remote_image_to_theme(url: &str) -> Result<style::core::color::theme::Theme, String> {
    let resp = reqwest::get(url)
        .await
        .map_err(|e| format!("request failed: {}", e))?;
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("bytes read failed: {}", e))?;

    let mut img = style::core::color::image::ImageReader::read(bytes.to_vec())
        .map_err(|e| format!("failed to decode image: {}", e))?;

    use style::core::color::image::FilterType;
    img.resize(128, 128, FilterType::Lanczos3);

    let source = style::core::color::image::ImageReader::extract_color(&img);

    Ok(ThemeBuilder::with_source(source).build())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let color_theme = color_to_theme(Argb::from_u32(0xffaae5a4));
    println!("[");
    println!("{},", serde_json::to_string_pretty(&color_theme)?);

    match local_image_to_theme("media/suzume-no-tojimari.png") {
        Ok(theme) => {
            println!("{},", serde_json::to_string_pretty(&theme)?);
        }
        Err(e) => {
            println!("{} {}", "Local image theme failed:".red(), e);
        }
    }

    let remote_url = "https://picsum.photos/id/866/1920/1080";

    match remote_image_to_theme(remote_url).await {
        Ok(theme) => {
            println!("{}", serde_json::to_string_pretty(&theme)?);
            println!("]");
        }
        Err(e) => {
            println!("{} {}", "Remote image theme failed:".red(), e);
        }
    }

    Ok(())
}

```

## Coverage / Parity Tool

## Dynamic Color Utilities

Any class starting with `bg-` or `text-` automatically generates a CSS variable and utility rule.

Resolution order for `<token>` in `bg-<token>` / `text-<token>`:
1. If `<token>` exists in `colors.toml`, that value is used.
2. Else if `<token>` is a standard CSS color keyword (e.g. `rebeccapurple`, `transparent`, `currentcolor`).
3. Else if `<token>` matches hex shorthand or long form without `#` (3/4/6/8 hex digits), it is interpreted as `#<token>`.

Generated artifacts:
- Variable: `--color-<token>` in `:root` and `.dark` scopes.
- Utility: `.bg-<token> { background-color: var(--color-<token>) }` or `.text-<token> { color: var(--color-<token>) }`.

Examples:
```
bg-ff0          => --color-ff0: #ff0;
text-112233cc   => --color-112233cc: #112233cc;
bg-rebeccapurple => --color-rebeccapurple: rebeccapurple;
text-brand      (if brand in colors.toml)
```

Invalid tokens (not in colors.toml, not keyword, not hex) are ignored silently.

The `coverage` binary compares configured utilities & variants to a Tailwind spec snapshot (no `tw-` prefix) for gap analysis.

Run examples:

```bash
cargo run --bin coverage
cargo run --bin coverage -- --json > coverage.json
cargo run --bin coverage -- --only-category sizing
cargo run --bin coverage -- --fail-on-missing any
```

Flags:

- `--json` : Machine-readable JSON output (suppresses text mode).
- `--only-category <name>` : Limit category coverage output / JSON categories subset.
- `--hide-missing-utilities` : Suppress listing of missing utility keys (text mode).
- `--hide-missing-values` : Suppress per-value missing suffix details (text mode).
- `--fail-on-missing [utilities|values|variants|any]` : Exit 1 if gaps; default `any` when provided without argument.
- `--generator-covers a,b,c` : (Optional) Explicitly mark value sets as fully covered when numeric generator present. Normally auto-detected; use to override.

Auto-detection:

Sizing, spacing, radii, line-height, and logical size value sets are automatically considered covered when a numeric generator is found, removing the need for manual flags in most cases.

Spec expansion:

Base spec lives in `tailwind-spec.json`. Additional spec fragments (utilities or variants) can be merged by dropping JSON files into `spec-extra/`. These files may contain:

```json
{
    "utilities": { "sizing": ["w","h"], "layout": ["flex"] },
    "variants": ["hover","focus"]
}
```

Variant matrix:

JSON output includes a `variants` object with present/missing/extra plus a `matrix_counts` map (heuristic counts of implemented prefixed utilities). Container query variants are not synthesized automatically; only variants explicitly defined in your TOML/state configuration appear.

Arbitrary values:

Bracketed classes (e.g. `w-[3.7rem]`) are detected and counted but excluded from value diffs.

CI usage example (fail build if any gap):

```bash
cargo run --quiet --bin coverage -- --fail-on-missing any
```

Or only enforce variants:

```bash
cargo run --quiet --bin coverage -- --fail-on-missing variants
```

> Note: Missing utility counts depend on how complete your spec files are. Expand `tailwind-spec.json` / `spec-extra` to reduce false positives.

```rust
use serde_json;
use std::fs::File;
use std::hash::Hasher;
use std::path::Path;
use std::sync::{Arc, Mutex};

mod cache;
mod config;
mod core;
mod datasource;
mod generator;
mod parser;
mod telemetry;
mod watcher;

use crate::config::Config;
use core::{AppState, rebuild_styles, set_base_layer_present, set_properties_layer_present};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load().unwrap_or_else(|_| Config::default());

    unsafe {
        std::env::set_var("DX_CACHE_DIR", config.resolved_cache_dir());
        let style_bin = format!("{}/style.bin", config.resolved_style_dir());
        std::env::set_var("DX_STYLE_BIN", style_bin);
    }

    if !Path::new(&config.paths.css_file).exists() {
        File::create(&config.paths.css_file)?;
    }
    if !Path::new(&config.paths.index_file).exists() {
        File::create(&config.paths.index_file)?;
    }

    if let Ok(val) = std::env::var("DX_MMAP_THRESHOLD") {
        if let Ok(parsed) = val.parse::<u64>() {
            core::output::set_mmap_threshold(parsed);
        }
    }
    let css_out = core::output::CssOutput::open(&config.paths.css_file)?;

    let (preloaded_cache, preloaded_hash, preloaded_checksum) = cache::load_cache();

    let existing_css_hash = {
        use ahash::AHasher;
        use std::io::Read;
        let mut hasher = AHasher::default();
        if let Ok(mut f) = std::fs::File::open(&config.paths.css_file) {
            let mut buf = [0u8; 8192];
            loop {
                match f.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => hasher.write(&buf[..n]),
                }
            }
            hasher.finish()
        } else {
            0
        }
    };

    let class_list_checksum = preloaded_checksum;
    let mut css_index = ahash::AHashMap::with_capacity(256);
    if let Ok(existing) = std::fs::read(&config.paths.css_file) {
        if existing.windows(11).any(|w| w == b"@layer base") {
            set_base_layer_present();
        }
        if existing.windows(18).any(|w| w == b"@layer properties") {
            set_properties_layer_present();
        }
        let mut offset = 0usize;
        for line in existing.split(|b| *b == b'\n') {
            let trimmed = {
                let mut i = 0;
                while i < line.len() && (line[i] == b' ' || line[i] == b'\t') {
                    i += 1;
                }
                &line[i..]
            };
            if trimmed.starts_with(b".") {
                if let Some(brace) = trimmed.iter().position(|c| *c == b'{') {
                    let cls = String::from_utf8_lossy(&trimmed[1..brace]).to_string();
                    let len = line.len() + 1;
                    css_index.insert(cls, (offset, len));
                    offset += len;
                } else {
                    offset += line.len() + 1;
                }
            } else {
                offset += line.len() + 1;
            }
        }
    }
    let app_state = Arc::new(Mutex::new(AppState {
        html_hash: preloaded_hash,
        class_cache: preloaded_cache,
        css_out,
        last_css_hash: existing_css_hash,
        css_buffer: Vec::with_capacity(8192),
        class_list_checksum,
        css_index,
        utilities_offset: 0,
    }));

    if std::env::var("DX_DUMP_STATE_ON_START").is_ok() {
        let s = app_state.lock().unwrap();
        let dump = serde_json::json!({
            "html_hash": s.html_hash,
            "class_cache_len": s.class_cache.len()
        });
        println!("{}", dump.to_string());
        return Ok(());
    }

    rebuild_styles(app_state.clone(), &config.paths.index_file, true)?;

    watcher::start(app_state, config)?;

    Ok(())
}
```
