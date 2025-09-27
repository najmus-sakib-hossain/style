use crate::{
    cache, datasource, generator,
    parser::{extract_classes_fast, rewrite_duplicate_classes},
    telemetry::format_duration,
};
mod animation;
mod engine;
mod formatter;
pub mod group;
use ahash::{AHashMap, AHashSet, AHasher};
use colored::Colorize;
use std::borrow::Cow;
use std::hash::Hasher;
pub mod color;
pub mod output;
use cssparser::serialize_identifier;
use output::CssOutput;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

static BASE_LAYER_PRESENT: AtomicBool = AtomicBool::new(false);
pub fn set_base_layer_present() {
    BASE_LAYER_PRESENT.store(true, Ordering::Relaxed);
}
#[allow(dead_code)]
fn base_layer_present() -> bool {
    BASE_LAYER_PRESENT.load(Ordering::Relaxed)
}

static PROPERTIES_LAYER_PRESENT: AtomicBool = AtomicBool::new(false);
pub fn set_properties_layer_present() {
    PROPERTIES_LAYER_PRESENT.store(true, Ordering::Relaxed);
}
#[allow(dead_code)]
pub fn properties_layer_present() -> bool {
    PROPERTIES_LAYER_PRESENT.load(Ordering::Relaxed)
}

static FIRST_LOG_DONE: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug)]
pub struct RuleMeta {
    pub off: usize, // offset relative to utilities body start
    pub len: usize, // full rule span including all lines and trailing newline
}

pub struct AppState {
    pub html_hash: u64,
    pub class_cache: AHashSet<String>,
    pub css_out: CssOutput,
    pub last_css_hash: u64,
    pub css_buffer: Vec<u8>,
    pub class_list_checksum: u64,
    pub css_index: ahash::AHashMap<String, RuleMeta>,
    pub utilities_offset: usize,
    pub group_registry: group::GroupRegistry,
    pub group_log_hash: u64,
}

impl AppState {
    pub fn engine() -> &'static engine::StyleEngine {
        use std::sync::OnceLock;
        static INSTANCE: OnceLock<engine::StyleEngine> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            engine::StyleEngine::load_from_disk().unwrap_or_else(|_| engine::StyleEngine::empty())
        })
    }
}

pub fn rebuild_styles(
    state: Arc<Mutex<AppState>>,
    index_path: &str,
    is_initial_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut html_bytes = datasource::read_file(index_path)?;
    let mut dev_group_selectors: AHashMap<String, String> = AHashMap::default();
    if let Some(plan) = rewrite_duplicate_classes(&html_bytes) {
        if !plan.groups.is_empty() {
            for info in &plan.groups {
                println!(
                    "[dx-style] auto group {} -> {}",
                    info.alias,
                    info.classes.join(" ")
                );
                dev_group_selectors.insert(
                    info.alias.clone(),
                    format!("@{}({})", info.alias, info.classes.join(" ")),
                );
            }
        }
        if plan.html != html_bytes {
            std::fs::write(index_path, &plan.html)?;
            html_bytes = plan.html;
        }
    }

    let hash_timer = Instant::now();
    let new_html_hash = {
        let mut hasher = AHasher::default();
        hasher.write(&html_bytes);
        hasher.finish()
    };
    let hash_duration = hash_timer.elapsed();

    let force_full = std::env::var("DX_FORCE_FULL").ok().as_deref() == Some("1")
        || std::env::var("DX_FORCE_FORMAT").ok().as_deref() == Some("1");
    {
        let state_guard = state.lock().unwrap();
        let html_same = state_guard.html_hash == new_html_hash;
        let css_complete = state_guard.css_index.len() == state_guard.class_cache.len();
        if !force_full && html_same && (!is_initial_run || css_complete) {
            return Ok(());
        }
    }

    let parse_timer = Instant::now();
    let prev_len_hint = { state.lock().unwrap().class_cache.len() };
    let extracted = extract_classes_fast(&html_bytes, prev_len_hint.next_power_of_two());
    let parse_extract_duration = parse_timer.elapsed();

    let mut all_classes = extracted.classes;
    let mut group_registry = group::GroupRegistry::analyze(
        &extracted.group_events,
        &mut all_classes,
        Some(AppState::engine()),
    );
    group_registry.set_dev_selectors(dev_group_selectors);

    let diff_timer = Instant::now();
    let (added, removed) = {
        let state_guard = state.lock().unwrap();
        let old = &state_guard.class_cache;
        let added: Vec<String> = all_classes.difference(old).cloned().collect();
        let removed: Vec<String> = old.difference(&all_classes).cloned().collect();
        (added, removed)
    };
    let diff_duration = diff_timer.elapsed();

    let css_incomplete = {
        let s = state.lock().unwrap();
        s.css_index.len() != s.class_cache.len()
    };
    // When forced formatting is requested (auto-format pass), we must not early-return
    // even if there are no class additions/removals, so that the pretty formatter runs.
    let force_format = std::env::var("DX_FORCE_FORMAT").ok().as_deref() == Some("1");
    if !force_format && added.is_empty() && removed.is_empty() && !css_incomplete {
        let mut state_guard = state.lock().unwrap();
        let mut h = AHasher::default();
        for c in &state_guard.class_cache {
            h.write(c.as_bytes());
        }
        state_guard.class_list_checksum = h.finish();
        state_guard.html_hash = new_html_hash;
        return Ok(());
    }

    let cache_update_timer = Instant::now();
    {
        let mut state_guard = state.lock().unwrap();
        state_guard.html_hash = new_html_hash;
        state_guard.class_cache = all_classes.clone();
        let mut h = AHasher::default();
        for c in &state_guard.class_cache {
            h.write(c.as_bytes());
        }
        state_guard.class_list_checksum = h.finish();
        state_guard.group_registry = group_registry;
        if state_guard.group_registry.is_empty() {
            state_guard.group_log_hash = 0;
        } else {
            let mut entries: Vec<(String, Vec<String>, bool)> = state_guard
                .group_registry
                .definitions()
                .map(|(name, def)| (name.clone(), def.utilities.clone(), def.allow_extend))
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let mut log_hasher = AHasher::default();
            for (name, utils, extend) in &entries {
                log_hasher.write(name.as_bytes());
                log_hasher.write(&[*extend as u8]);
                for util in utils {
                    log_hasher.write(util.as_bytes());
                }
            }
            let new_hash = log_hasher.finish();
            if new_hash != state_guard.group_log_hash {
                for (name, utils, extend) in &entries {
                    let mut message = format!("[dx-style] group {} -> {}", name, utils.join(" "));
                    if *extend {
                        message.push_str(" (extend)");
                    }
                    println!("{}", message);
                }
                state_guard.group_log_hash = new_hash;
            }
        }
    }
    let cache_update_duration = cache_update_timer.elapsed();

    if let Err(e) = cache::save_cache(&state.lock().unwrap().class_cache, new_html_hash) {
        eprintln!("{} {}", "Error saving cache:".red(), e);
    }

    struct WriteStats {
        mode: &'static str,
        classes_written: usize,
        bytes_written: usize,
        sub1_label: &'static str,
        sub1: std::time::Duration,
        sub2_label: Option<&'static str>,
        sub2: Option<std::time::Duration>,
        sub3_label: Option<&'static str>,
        sub3: Option<std::time::Duration>,
    }
    let css_write_timer = Instant::now();
    let (css_write_duration, write_stats) = {
        let mut state_guard = state.lock().unwrap();
        state_guard.css_buffer.clear();
        let is_color = |c: &str| {
            let base = c.rsplit(':').next().unwrap_or(c);
            base.starts_with("bg-") || base.starts_with("text-")
        };
        let removed_has_color = removed.iter().any(|c| is_color(c));
        let added_has_color = added.iter().any(|c| is_color(c));
        let missing_index_for_removed = removed
            .iter()
            .any(|c| !state_guard.css_index.contains_key(c));
        let only_additions = !added.is_empty() && removed.is_empty();
        let only_removals = !removed.is_empty() && added.is_empty();
        let need_full = if force_full || is_initial_run {
            true
        } else if only_additions {
            added_has_color
        } else if only_removals {
            removed_has_color || missing_index_for_removed
        } else {
            true
        };
        if need_full {
            let mut class_vec: Vec<String> = state_guard.class_cache.iter().cloned().collect();
            class_vec.sort();
            let phase_start = Instant::now();
            state_guard
                .css_buffer
                .extend_from_slice(b"@layer theme, components, base, properties, utilities;\n");
            fn write_layer(buf: &mut Vec<u8>, name: &str, body: &str) {
                let trimmed = body.trim();
                if trimmed.is_empty() {
                    buf.extend_from_slice(format!("@layer {} {{}}\n", name).as_bytes());
                } else {
                    buf.extend_from_slice(format!("@layer {} {{\n", name).as_bytes());
                    for line in trimmed.lines() {
                        if line.is_empty() {
                            continue;
                        }
                        buf.extend_from_slice(b"  ");
                        buf.extend_from_slice(line.as_bytes());
                        buf.push(b'\n');
                    }
                    buf.extend_from_slice(b"}\n");
                }
            }
            let (root_vars, dark_vars) = {
                let engine = AppState::engine();
                engine.generate_color_vars_for(
                    class_vec.iter().collect::<Vec<_>>().iter().map(|s| *s),
                )
            };
            let mut theme_body = String::new();
            if !root_vars.is_empty() {
                theme_body.push_str(root_vars.trim_end());
                theme_body.push('\n');
            }
            if !dark_vars.is_empty() {
                theme_body.push_str(dark_vars.trim_end());
                theme_body.push('\n');
            }
            write_layer(&mut state_guard.css_buffer, "theme", &theme_body);
            write_layer(&mut state_guard.css_buffer, "components", "");
            if let Some(base_raw) = AppState::engine().base_layer_raw.as_ref() {
                if !base_raw.is_empty() {
                    let mut base_body = String::new();
                    for line in base_raw.trim_end().lines() {
                        if line.trim().is_empty() {
                            continue;
                        }
                        base_body.push_str(line);
                        base_body.push('\n');
                    }
                    write_layer(&mut state_guard.css_buffer, "base", &base_body);
                } else {
                    write_layer(&mut state_guard.css_buffer, "base", "");
                }
            } else {
                write_layer(&mut state_guard.css_buffer, "base", "");
            }
            set_base_layer_present();
            {
                let engine = AppState::engine();
                // Prefer explicit property layer raw content if provided.
                let mut prop_body = if let Some(prop_raw) = engine.property_layer_raw.as_ref() {
                    if prop_raw.trim().is_empty() {
                        String::new()
                    } else {
                        let mut b = String::new();
                        for line in prop_raw.lines() {
                            if line.trim().is_empty() {
                                continue;
                            }
                            b.push_str(line);
                            b.push('\n');
                        }
                        b
                    }
                } else {
                    String::new()
                };
                // Fallback: if still empty, synthesize from registered @property at-rules.
                if prop_body.is_empty() {
                    let at_rules = engine.property_at_rules();
                    if !at_rules.trim().is_empty() {
                        prop_body.push_str(at_rules.trim_end());
                        prop_body.push('\n');
                    }
                }
                write_layer(&mut state_guard.css_buffer, "properties", &prop_body);
                set_properties_layer_present();
            }

            let mut util_buf = Vec::new();
            generator::generate_class_rules_only(
                &mut util_buf,
                class_vec.iter(),
                &mut state_guard.group_registry,
            );
            let gen_layers_utils = phase_start.elapsed();
            let util_phase_start = Instant::now();
            let mut util_body = String::new();
            for line in String::from_utf8_lossy(&util_buf).lines() {
                if line.trim().is_empty() {
                    continue;
                }
                util_body.push_str(line);
                util_body.push('\n');
            }
            state_guard
                .css_buffer
                .extend_from_slice(b"@layer utilities {\n");
            state_guard.utilities_offset = state_guard.css_buffer.len();
            for line in util_body.lines() {
                if line.is_empty() {
                    continue;
                }
                state_guard.css_buffer.extend_from_slice(b"  ");
                state_guard.css_buffer.extend_from_slice(line.as_bytes());
                state_guard.css_buffer.push(b'\n');
            }
            state_guard.css_buffer.extend_from_slice(b"}\n");
            // Always format CSS output to maintain pretty-printed styling for downstream tools.
            if let Ok(as_string) = String::from_utf8(state_guard.css_buffer.clone()) {
                if let Some(formatted) = formatter::format_css_pretty(&as_string) {
                    state_guard.css_buffer.clear();
                    state_guard
                        .css_buffer
                        .extend_from_slice(formatted.as_bytes());
                    // Recompute utilities_offset since formatting may have changed positions
                    if let Some(layer_pos) =
                        twoway::find_bytes(&state_guard.css_buffer, b"@layer utilities")
                    {
                        // Find first '{' after the marker
                        if let Some(rel_brace) = state_guard.css_buffer[layer_pos..]
                            .iter()
                            .position(|b| *b == b'{')
                        {
                            let after_brace = layer_pos + rel_brace + 1; // position right after '{'
                            // Advance to next newline (start of body lines)
                            if let Some(nl) = state_guard.css_buffer[after_brace..]
                                .iter()
                                .position(|b| *b == b'\n')
                            {
                                state_guard.utilities_offset = after_brace + nl + 1;
                            } else {
                                // No newline -> body empty so offset at end
                                state_guard.utilities_offset = state_guard.css_buffer.len();
                            }
                        }
                    }
                }
            }
            let fragment_vec = state_guard.css_buffer.clone();
            let build_utilities = util_phase_start.elapsed();
            let flush_start = Instant::now();
            use ahash::AHasher;
            let mut hh = AHasher::default();
            hh.write(&fragment_vec);
            let frag_hash = hh.finish();
            let fragment_len = fragment_vec.len();
            let utilities_offset = state_guard.utilities_offset;
            let mut wrote = false;
            if state_guard.last_css_hash != frag_hash {
                // Content changed after formatting -> always write.
                state_guard.css_out.replace(&fragment_vec)?;
                state_guard.last_css_hash = frag_hash;
                wrote = true;
            } else if std::env::var("DX_FORCE_FULL").ok().as_deref() == Some("1") {
                // force_write explicitly requested via config (full rewrite every pass)
                state_guard.css_out.replace(&fragment_vec)?;
                state_guard.last_css_hash = frag_hash; // unchanged but re-written
                wrote = true;
            }
            if force_format {
                unsafe {
                    if wrote {
                        std::env::set_var("DX_FORMAT_STATUS", "rewritten");
                    } else {
                        std::env::set_var("DX_FORMAT_STATUS", "unchanged");
                    }
                }
            }
            state_guard.css_index.clear();
            // Utilities body without the final closing "}\n" (2 bytes) of the layer.
            if utilities_offset >= fragment_len
                || fragment_len < 2
                || utilities_offset + 2 > fragment_len
            {
                // Nothing indexable; avoid panic
                state_guard.css_out.flush_now()?;
                let flush_time = flush_start.elapsed();
                (
                    css_write_timer.elapsed(),
                    WriteStats {
                        mode: "full",
                        classes_written: class_vec.len(),
                        bytes_written: fragment_vec.len(),
                        sub1_label: "layers+gen",
                        sub1: gen_layers_utils,
                        sub2_label: Some("utilities"),
                        sub2: Some(build_utilities),
                        sub3_label: Some("flush"),
                        sub3: Some(flush_time),
                    },
                );
                // Early continue of outer block by using a label is complicated; we just fall through with empty index
                // Return from closure style requires restructure; keeping simple no indexing.
            }
            let body_slice: &[u8] = if utilities_offset + 2 <= fragment_len {
                &fragment_vec[utilities_offset..fragment_len - 2]
            } else {
                &[]
            };
            let mut cursor = 0usize;
            while cursor < body_slice.len() {
                // Skip leading whitespace/newlines
                while cursor < body_slice.len()
                    && (body_slice[cursor] == b'\n'
                        || body_slice[cursor] == b' '
                        || body_slice[cursor] == b'\t')
                {
                    cursor += 1;
                }
                if cursor >= body_slice.len() {
                    break;
                }
                let rule_start = cursor; // potential start
                if body_slice[cursor] == b'.' {
                    // Read selector until '{'
                    let mut sel_end = cursor + 1;
                    while sel_end < body_slice.len()
                        && body_slice[sel_end] != b'{'
                        && body_slice[sel_end] != b'\n'
                    {
                        sel_end += 1;
                    }
                    if sel_end < body_slice.len() && body_slice[sel_end] == b'{' {
                        // Extract class name between '.' and first '{'
                        let raw = &body_slice[cursor + 1..sel_end];
                        let mut end_trim = raw.len();
                        while end_trim > 0
                            && (raw[end_trim - 1] == b' ' || raw[end_trim - 1] == b'\t')
                        {
                            end_trim -= 1;
                        }
                        if end_trim > 0 {
                            let name = String::from_utf8_lossy(&raw[..end_trim]).to_string();
                            if !name.is_empty() {
                                // Balance braces to find end of rule
                                let mut depth: isize = 0;
                                let mut j = sel_end;
                                let mut rule_end = sel_end;
                                while j < body_slice.len() {
                                    let b = body_slice[j];
                                    if b == b'{' {
                                        depth += 1;
                                    }
                                    if b == b'}' {
                                        depth -= 1;
                                    }
                                    if depth == 0 && b == b'}' {
                                        // consume to end-of-line
                                        let mut k = j;
                                        while k < body_slice.len() && body_slice[k] != b'\n' {
                                            k += 1;
                                        }
                                        if k < body_slice.len() {
                                            k += 1;
                                        }
                                        rule_end = k;
                                        break;
                                    }
                                    j += 1;
                                }
                                if rule_end > rule_start {
                                    state_guard.css_index.insert(
                                        name,
                                        RuleMeta {
                                            off: rule_start,
                                            len: rule_end - rule_start,
                                        },
                                    );
                                    cursor = rule_end;
                                    continue;
                                }
                            }
                        }
                    }
                }
                // Fallback: advance to next newline if not a rule
                while cursor < body_slice.len() && body_slice[cursor] != b'\n' {
                    cursor += 1;
                }
                if cursor < body_slice.len() {
                    cursor += 1;
                }
            }
            state_guard.css_out.flush_now()?;
            let flush_time = flush_start.elapsed();
            (
                css_write_timer.elapsed(),
                WriteStats {
                    mode: "full",
                    classes_written: class_vec.len(),
                    bytes_written: fragment_vec.len(),
                    sub1_label: "layers+gen",
                    sub1: gen_layers_utils,
                    sub2_label: Some("utilities"),
                    sub2: Some(build_utilities),
                    sub3_label: Some("flush"),
                    sub3: Some(flush_time),
                },
            )
        } else if only_additions {
            // Fast incremental add: build block and index without rescanning bytes
            let gen_start = Instant::now();
            let engine = AppState::engine();
            let mut block: Vec<u8> = Vec::new();
            block.push(b'\n');
            // Store temporary offsets relative to start of block
            let mut offsets: Vec<(String, usize, usize)> = Vec::with_capacity(added.len());
            let mut cursor_in_block = 1usize; // account for leading '\n'
            let mut escaped = String::with_capacity(64);
            for class in &added {
                if state_guard.group_registry.is_internal_token(class) {
                    continue;
                }
                let css_cow: Cow<'_, str> = if let Some(alias_css) =
                    state_guard.group_registry.generate_css_for(class, engine)
                {
                    Cow::Borrowed(alias_css)
                } else if let Some(css) = engine.css_for_class(class) {
                    Cow::Owned(css)
                } else {
                    escaped.clear();
                    serialize_identifier(class, &mut escaped).unwrap();
                    Cow::Owned(format!(".{} {{}}\n", escaped))
                };
                let mut css = css_cow.as_ref().to_string();
                if css.trim().is_empty() {
                    continue;
                }
                if !css.ends_with('\n') {
                    css.push('\n');
                }
                // Record start of rule at '.' (skip the two-space indent)
                let rule_start_block = cursor_in_block + 2;
                // Write indented lines
                for line in css.lines() {
                    if line.is_empty() {
                        continue;
                    }
                    block.extend_from_slice(b"  ");
                    block.extend_from_slice(line.as_bytes());
                    block.push(b'\n');
                    cursor_in_block += 2 + line.len() + 1; // indent + line + newline
                }
                let rule_len = cursor_in_block.saturating_sub(rule_start_block);
                offsets.push((class.clone(), rule_start_block, rule_len));
            }
            let classes_written = offsets.len();
            let gen_time = gen_start.elapsed();
            let build_time = std::time::Duration::from_micros(0);
            let flush_start = Instant::now();
            let start_rel = state_guard.css_out.append_inside_final_block(&block)?;
            let rel_base = start_rel - state_guard.utilities_offset;
            for (name, off_blk, len) in offsets {
                state_guard.css_index.insert(
                    name,
                    RuleMeta {
                        off: rel_base + off_blk,
                        len,
                    },
                );
            }
            state_guard.css_out.flush_now()?;
            let flush_time = flush_start.elapsed();
            (
                css_write_timer.elapsed(),
                WriteStats {
                    mode: "add",
                    classes_written,
                    bytes_written: block.len(),
                    sub1_label: "gen",
                    sub1: gen_time,
                    sub2_label: Some("build"),
                    sub2: Some(build_time),
                    sub3_label: Some("flush"),
                    sub3: Some(flush_time),
                },
            )
        } else if !removed.is_empty() && added.is_empty() {
            let mut removed_bytes = 0usize;
            let mut removed_count = 0usize;
            for r in &removed {
                if state_guard.group_registry.is_internal_token(r) {
                    continue;
                }
                if let Some(meta) = state_guard.css_index.remove(r) {
                    let rel = state_guard.utilities_offset + meta.off;
                    let _ = state_guard.css_out.blank_range(rel, meta.len);
                    removed_bytes += meta.len;
                    removed_count += 1;
                }
            }
            let flush_start = Instant::now();
            state_guard.css_out.flush_now()?;
            let flush_time = flush_start.elapsed();
            (
                css_write_timer.elapsed(),
                WriteStats {
                    mode: "remove",
                    classes_written: removed_count,
                    bytes_written: removed_bytes,
                    sub1_label: "blank",
                    sub1: flush_time,
                    sub2_label: None,
                    sub2: None,
                    sub3_label: None,
                    sub3: None,
                },
            )
        } else {
            drop(state_guard);
            (
                css_write_timer.elapsed(),
                WriteStats {
                    mode: "mixed",
                    classes_written: added.len() + removed.len(),
                    bytes_written: 0,
                    sub1_label: "noop",
                    sub1: std::time::Duration::from_micros(0),
                    sub2_label: None,
                    sub2: None,
                    sub3_label: None,
                    sub3: None,
                },
            )
        }
    };

    let total_processing = hash_duration
        + parse_extract_duration
        + diff_duration
        + cache_update_duration
        + css_write_duration;

    let silent_format = std::env::var("DX_SILENT_FORMAT").ok().as_deref() == Some("1");
    if !silent_format && !FIRST_LOG_DONE.load(Ordering::Relaxed) {
        // For the very first run, also show full timing details like subsequent runs.
        let mut write_detail = format!(
            "mode={} classes={} bytes={} {}={:?}",
            write_stats.mode,
            write_stats.classes_written,
            write_stats.bytes_written,
            write_stats.sub1_label,
            write_stats.sub1
        );
        if let (Some(l2), Some(d2)) = (write_stats.sub2_label, write_stats.sub2) {
            write_detail.push_str(&format!(" {}={:?}", l2, d2));
        }
        if let (Some(l3), Some(d3)) = (write_stats.sub3_label, write_stats.sub3) {
            write_detail.push_str(&format!(" {}={:?}", l3, d3));
        }
        println!(
            "Initial: {} added, {} removed | (Total: {} -> Hash: {}, Parse: {}, Diff: {}, Cache: {}, Write: {} [{}])",
            format!("{}", added.len()).green(),
            format!("{}", removed.len()).red(),
            format_duration(total_processing),
            format_duration(hash_duration),
            format_duration(parse_extract_duration),
            format_duration(diff_duration),
            format_duration(cache_update_duration),
            format_duration(css_write_duration),
            write_detail
        );
        FIRST_LOG_DONE.store(true, Ordering::Relaxed);
    } else if !silent_format {
        // Build write detail string
        let mut write_detail = format!(
            "mode={} classes={} bytes={} {}={:?}",
            write_stats.mode,
            write_stats.classes_written,
            write_stats.bytes_written,
            write_stats.sub1_label,
            write_stats.sub1
        );
        if let (Some(l2), Some(d2)) = (write_stats.sub2_label, write_stats.sub2) {
            write_detail.push_str(&format!(" {}={:?}", l2, d2));
        }
        if let (Some(l3), Some(d3)) = (write_stats.sub3_label, write_stats.sub3) {
            write_detail.push_str(&format!(" {}={:?}", l3, d3));
        }
        println!(
            "Processed: {} added, {} removed | (Total: {} -> Hash: {}, Parse: {}, Diff: {}, Cache: {}, Write: {} [{}])",
            format!("{}", added.len()).green(),
            format!("{}", removed.len()).red(),
            format_duration(total_processing),
            format_duration(hash_duration),
            format_duration(parse_extract_duration),
            format_duration(diff_duration),
            format_duration(cache_update_duration),
            format_duration(css_write_duration),
            write_detail
        );
    }

    if !added.is_empty() || !removed.is_empty() {
        if let Ok(mut guard) = state.lock() {
            let _ = guard.css_out.flush_now();
        }
    }

    Ok(())
}
