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
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use std::thread;
use std::time::{Duration, Instant};

fn start_delayed_formatter(
    state: Arc<Mutex<AppState>>,
    index_path: String,
    initial_delay_ms: u64,
    interval_ms: u64,
    debounce_ms: u64,
    force_write: bool,
) {
    if interval_ms == 0 {
        return;
    }
    rayon::spawn(move || {
        if initial_delay_ms > 0 {
            std::thread::sleep(Duration::from_millis(initial_delay_ms));
        }
        let mut last_classes_checksum: u64 = 0;
        let mut last_run = Instant::now() - Duration::from_millis(interval_ms);
        loop {
            let (current_checksum, time_ok) = {
                if let Ok(g) = state.lock() {
                    (
                        g.class_list_checksum,
                        last_run.elapsed() >= Duration::from_millis(interval_ms),
                    )
                } else {
                    (0, false)
                }
            };
            if current_checksum == last_classes_checksum && !time_ok {
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
            if current_checksum != last_classes_checksum {
                let changed_at = Instant::now();
                while changed_at.elapsed() < Duration::from_millis(debounce_ms) {
                    let new_checksum = {
                        state
                            .lock()
                            .ok()
                            .map(|s| s.class_list_checksum)
                            .unwrap_or(current_checksum)
                    };
                    if new_checksum != current_checksum {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }
            }
            last_run = Instant::now();
            last_classes_checksum = current_checksum;
            let start = Instant::now();
            unsafe {
                std::env::set_var("DX_FORCE_FORMAT", "1");
                std::env::set_var("DX_SILENT_FORMAT", "1");
                if force_write {
                    std::env::set_var("DX_FORCE_FULL", "1");
                }
            }
            let _ = rebuild_styles(state.clone(), &index_path, false);
            unsafe {
                std::env::remove_var("DX_FORCE_FORMAT");
                std::env::remove_var("DX_SILENT_FORMAT");
                if force_write {
                    std::env::remove_var("DX_FORCE_FULL");
                }
            }
            let _dur = start.elapsed();
            std::thread::sleep(Duration::from_millis(50));
        }
    });
}

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

    let (preloaded_cache, preloaded_hash, preloaded_checksum, preloaded_groups) =
        cache::load_cache();

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
        let mut cursor = 0usize;
        while cursor < existing.len() {
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
        group_registry: if let Some(dump) = preloaded_groups {
            crate::core::group::GroupRegistry::from_dump(&dump)
        } else {
            crate::core::group::GroupRegistry::new()
        },
        group_log_hash: 0,
        incremental_parser: crate::parser::IncrementalParser::new(),
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
    start_delayed_formatter(
        app_state.clone(),
        config.paths.index_file.clone(),
        config.format_delay_ms(),
        config.format_interval_ms(),
        config.format_debounce_ms(),
        config.format_force_write(),
    );

    start_css_validator(app_state.clone());

    watcher::start(app_state, config)?;

    Ok(())
}

fn start_css_validator(state: Arc<Mutex<AppState>>) {
    const INTERVAL: Duration = Duration::from_millis(1500);
    thread::spawn(move || {
        let mut last_check = Instant::now() - INTERVAL;
        loop {
            if last_check.elapsed() < INTERVAL {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
            last_check = Instant::now();
            let path = {
                let guard = state.lock().ok();
                if let Some(g) = guard {
                    g.css_out.path().to_string()
                } else {
                    continue;
                }
            };
            let Ok(contents) = std::fs::read(&path) else {
                continue;
            };
            let Ok(text) = String::from_utf8(contents) else {
                continue;
            };
            if text.trim().is_empty() {
                continue;
            }
            let mut strict_ok = true;
            if StyleSheet::parse(
                &text,
                ParserOptions {
                    error_recovery: false,
                    ..ParserOptions::default()
                },
            )
            .is_err()
            {
                strict_ok = false;
            }
            let layer_names = ["theme", "components", "base", "properties", "utilities"];
            let mut layer_ranges: Vec<(usize, usize)> = Vec::new();
            let bytes = text.as_bytes();
            let mut i = 0usize;
            while i < bytes.len() {
                if bytes[i] == b'@' {
                    if i + 7 < bytes.len() && &bytes[i..i + 7] == b"@layer " {
                        let mut j = i + 7;
                        while j < bytes.len() && bytes[j].is_ascii_alphanumeric() {
                            j += 1;
                        }
                        let name = &text[i + 7..j];
                        if layer_names.iter().any(|ln| *ln == name) {
                            while j < bytes.len() && bytes[j] != b'{' {
                                j += 1;
                            }
                            if j >= bytes.len() {
                                break;
                            }
                            let mut depth: isize = 0;
                            let mut k = j;
                            while k < bytes.len() {
                                if bytes[k] == b'{' {
                                    depth += 1;
                                } else if bytes[k] == b'}' {
                                    depth -= 1;
                                    if depth == 0 {
                                        k += 1;
                                        break;
                                    }
                                }
                                k += 1;
                            }
                            layer_ranges.push((i, k.min(bytes.len())));
                            i = k;
                            continue;
                        }
                    }
                }
                i += 1;
            }
            let validator_log = std::env::var("DX_VALIDATOR_LOG")
                .ok()
                .map(|v| v == "1")
                .unwrap_or(false);
            if strict_ok {
                if let Some((_, last_end)) = layer_ranges.iter().max_by_key(|r| r.1) {
                    let trailing = &text[*last_end..];
                    if trailing.trim().contains('\n')
                        || trailing.trim().starts_with('.')
                        || trailing.contains('{')
                    {}
                }
                continue;
            }
            let is_generator_error = !layer_ranges.is_empty();
            if is_generator_error {
                if validator_log {
                    eprintln!("[validator] generator error -> forcing rebuild");
                }
                unsafe {
                    std::env::set_var("DX_FORCE_FORMAT", "1");
                }
                let index_file = if let Ok(cfg) = crate::config::Config::load() {
                    cfg.paths.index_file
                } else {
                    "index.html".to_string()
                };
                let _ = rebuild_styles(state.clone(), &index_file, true);
                unsafe {
                    std::env::remove_var("DX_FORCE_FORMAT");
                }
            } else {
                if validator_log {
                    eprintln!("[validator] user manual error -> commenting out invalid part");
                }
                if let Some(pos) = text.rfind('}') {
                    let (good, bad) = text.split_at(pos + 1);
                    if !bad.trim().is_empty() {
                        let mut commented = String::from(good);
                        commented.push_str("\n/* Invalid user CSS commented out:\n");
                        for line in bad.lines() {
                            if !line.trim().is_empty() {
                                commented.push_str(" * ");
                                commented.push_str(line);
                                commented.push('\n');
                            }
                        }
                        commented.push_str("*/\n");
                        if let Ok(mut guard) = state.lock() {
                            let _ = guard.css_out.replace(commented.as_bytes());
                            let _ = guard.css_out.flush_now();
                        }
                    }
                }
            }
        }
    });
}
