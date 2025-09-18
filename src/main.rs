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
        // Attempt multi-line rule recovery for existing utilities (best-effort).
        let mut cursor = 0usize;
        while cursor < existing.len() {
            // Skip whitespace/newlines
            while cursor < existing.len()
                && (existing[cursor] == b'\n'
                    || existing[cursor] == b' '
                    || existing[cursor] == b'\t')
            {
                cursor += 1;
            }
            if cursor >= existing.len() {
                break;
            }
            let start = cursor;
            if existing[cursor] == b'.' {
                let mut sel_end = cursor + 1;
                while sel_end < existing.len()
                    && existing[sel_end] != b'{'
                    && existing[sel_end] != b'\n'
                {
                    sel_end += 1;
                }
                if sel_end < existing.len() && existing[sel_end] == b'{' {
                    let raw = &existing[cursor + 1..sel_end];
                    let mut end_trim = raw.len();
                    while end_trim > 0 && (raw[end_trim - 1] == b' ' || raw[end_trim - 1] == b'\t')
                    {
                        end_trim -= 1;
                    }
                    if end_trim > 0 {
                        let name = String::from_utf8_lossy(&raw[..end_trim]).to_string();
                        if !name.is_empty() {
                            let mut depth: isize = 0;
                            let mut j = sel_end;
                            let mut rule_end = sel_end;
                            while j < existing.len() {
                                let b = existing[j];
                                if b == b'{' {
                                    depth += 1;
                                }
                                if b == b'}' {
                                    depth -= 1;
                                }
                                if depth == 0 && b == b'}' {
                                    let mut k = j;
                                    while k < existing.len() && existing[k] != b'\n' {
                                        k += 1;
                                    }
                                    if k < existing.len() {
                                        k += 1;
                                    }
                                    rule_end = k;
                                    break;
                                }
                                j += 1;
                            }
                            if rule_end > start {
                                css_index.insert(
                                    name,
                                    crate::core::RuleMeta {
                                        off: start,
                                        len: rule_end - start,
                                    },
                                );
                                cursor = rule_end;
                                continue;
                            }
                        }
                    }
                }
            }
            // Not a rule; move to next line
            while cursor < existing.len() && existing[cursor] != b'\n' {
                cursor += 1;
            }
            if cursor < existing.len() {
                cursor += 1;
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
