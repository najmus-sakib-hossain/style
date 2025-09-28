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
        // Only create dev selectors for groups that were written with an explicit
        // leading '@' (i.e. class="@alias(...)"). The rewrite function returns
        // group metadata for auto-generated alias writes; however some rewrites
        // (manual-expansion toggles) return groups empty. Respect that to allow
        // toggling the presence of the legacy dev selector.
        if !plan.groups.is_empty() {
            for info in &plan.groups {
                // println!(
                //     "[dx-style] auto group {} -> {}",
                //     info.alias,
                //     info.classes.join(" ")
                // );
                // Only insert dev selector if the group in HTML was emitted with '@'
                // (rewrite_duplicate_classes will record '@' in the replacement string
                // for the first occurrence). Detect this by searching plan.html for
                // the explicit @alias(...) substring.
                let search = format!("@{}(", info.alias);
                if String::from_utf8_lossy(&plan.html).contains(&search) {
                    dev_group_selectors.insert(
                        info.alias.clone(),
                        format!("@{}({})", info.alias, info.classes.join(" ")),
                    );
                }
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
    // New: scan the current HTML for grouped @alias(...) occurrences that refer
    // to alias names not present in the current registry (manual rename cases).
    // For each such occurrence, compute the set of inner utilities and try to
    // find the best matching current alias by Jaccard similarity; if a match
    // meets the threshold, rewrite the HTML to use the new alias and re-run
    // extraction/analysis.
    {
        let html_string_in = String::from_utf8_lossy(&html_bytes).to_string();
        // Regex to find @alias(inner...)
        let grouped_re = regex::Regex::new(r"@([A-Za-z0-9_\-]+)\s*\(\s*([^\)]*)\s*\)").unwrap();
        // Build normalized current defs for matching
        let mut current_alias_names: AHashSet<String> = AHashSet::default();
        let mut current_defs_norm: Vec<(String, AHashSet<String>)> = Vec::new();
        // Map of normalized (sorted) utility signature -> alias name for exact-match fallback
        let mut current_defs_map: AHashMap<String, String> = AHashMap::default();
        for (name, _def) in group_registry.definitions() {
            current_alias_names.insert(name.clone());
        }
        for (name, def) in group_registry.definitions() {
            let mut set: AHashSet<String> = AHashSet::default();
            for u in &def.utilities {
                if u.is_empty() {
                    continue;
                }
                if u.contains('@') {
                    continue;
                }
                if current_alias_names.contains(u) {
                    continue;
                }
                if group_registry.is_internal_token(u) {
                    continue;
                }
                set.insert(u.clone());
            }
            if !set.is_empty() {
                // build a deterministic signature for exact set matching
                let mut sig_vec: Vec<&str> = set.iter().map(|s| s.as_str()).collect();
                sig_vec.sort();
                let sig = sig_vec.join(" ");
                current_defs_map.insert(sig.clone(), name.clone());
                current_defs_norm.push((name.clone(), set));
            }
        }
        if !current_defs_norm.is_empty() {
            let threshold: f64 = std::env::var("DX_GROUP_RENAME_SIMILARITY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.6);
            let mut html_out = html_string_in.clone();
            let mut modified = false;
            for cap in grouped_re.captures_iter(&html_string_in) {
                let old_name = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                let inner = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                if old_name.is_empty() {
                    continue;
                }
                if current_alias_names.contains(&old_name) {
                    continue;
                }
                // Normalize inner tokens
                let mut old_set: AHashSet<String> = AHashSet::default();
                for tok in inner.split_whitespace() {
                    if tok.is_empty() {
                        continue;
                    }
                    if tok.contains('@') {
                        continue;
                    }
                    if current_alias_names.contains(tok) {
                        continue;
                    }
                    if group_registry.is_internal_token(tok) {
                        continue;
                    }
                    old_set.insert(tok.to_string());
                }
                if old_set.is_empty() {
                    continue;
                }
                // Find best (fuzzy) match
                let mut best_score = 0f64;
                let mut best_alias: Option<String> = None;
                for (cand_alias, cand_set) in &current_defs_norm {
                    let inter = old_set.iter().filter(|x| cand_set.contains(*x)).count();
                    let uni = old_set.len() + cand_set.len() - inter;
                    if uni == 0 {
                        continue;
                    }
                    let score = (inter as f64) / (uni as f64);
                    if score > best_score {
                        best_score = score;
                        best_alias = Some(cand_alias.clone());
                    }
                }
                // If fuzzy matching didn't find a confident candidate, try an exact-set
                // lookup based on the inner utilities (this fixes cases where the
                // current CSS/group registry already contains an exact alias for
                // the provided utilities but the fuzzy threshold was not met).
                if best_alias.is_none() {
                    let mut sig_vec: Vec<&str> = old_set.iter().map(|s| s.as_str()).collect();
                    sig_vec.sort();
                    let sig = sig_vec.join(" ");
                    if let Some(exact_alias) = current_defs_map.get(&sig) {
                        best_alias = Some(exact_alias.clone());
                        best_score = 1.0;
                        if std::env::var("DX_DEBUG").ok().as_deref() == Some("1") {
                            eprintln!(
                                "[dx-style-debug] exact-match fallback for @{} -> {} sig='{}'",
                                old_name, exact_alias, sig
                            );
                        }
                    }
                }
                if let Some(new_alias) = best_alias {
                    if best_score >= threshold && new_alias != old_name {
                        // Replace occurrences of @old( with @new(
                        let old_with_paren = format!("@{}(", old_name);
                        let new_with_paren = format!("@{}(", new_alias);
                        if html_out.contains(&old_with_paren) {
                            html_out = html_out.replace(&old_with_paren, &new_with_paren);
                            modified = true;
                        }
                        // Replace bare @old -> @new
                        let old_at = format!("@{}", old_name);
                        let new_at = format!("@{}", new_alias);
                        if html_out.contains(&old_at) {
                            html_out = html_out.replace(&old_at, &new_at);
                            modified = true;
                        }
                        // Also replace plain class tokens equal to old_name
                        let class_attr_re =
                            regex::Regex::new(r#"class\s*=\s*\"([^\"]*)\""#).unwrap();
                        let mut tmp_html = html_out.clone();
                        for cap2 in class_attr_re.captures_iter(&html_out) {
                            if let Some(m2) = cap2.get(0) {
                                let full = m2.as_str();
                                if let Some(group2) = cap2.get(1) {
                                    let classes_str = group2.as_str();
                                    let mut items: Vec<String> = classes_str
                                        .split_whitespace()
                                        .map(|s| s.to_string())
                                        .collect();
                                    let mut replaced_any = false;
                                    for it in items.iter_mut() {
                                        if it == &old_name {
                                            // Migrate plain token to plain alias (no leading '@')
                                            *it = new_alias.clone();
                                            replaced_any = true;
                                        }
                                    }
                                    if replaced_any {
                                        let new_attr = format!("class=\"{}\"", items.join(" "));
                                        tmp_html = tmp_html.replacen(full, &new_attr, 1);
                                    }
                                }
                            }
                        }
                        if tmp_html != html_out {
                            html_out = tmp_html;
                            modified = true;
                        }
                    }
                }
            }
            if modified {
                std::fs::write(index_path, &html_out)?;
                html_bytes = html_out.into_bytes();
                // Re-extract and re-analyze with rewritten HTML
                let extracted2 =
                    extract_classes_fast(&html_bytes, all_classes.len().next_power_of_two());
                let mut all_classes2 = extracted2.classes;
                group_registry = group::GroupRegistry::analyze(
                    &extracted2.group_events,
                    &mut all_classes2,
                    Some(AppState::engine()),
                );
                // update master class set
                all_classes = all_classes2;
            }
        }
    }
    // Detect alias renames using fuzzy Jaccard similarity of utility sets.
    // Exclude alias/internal tokens and nested aliases from comparisons.
    {
        let prev_registry = { state.lock().unwrap().group_registry.clone() };
        if prev_registry.definitions().next().is_some()
            && group_registry.definitions().next().is_some()
        {
            // Build current alias name set and normalized utility sets
            let mut current_alias_names: AHashSet<String> = AHashSet::default();
            for (name, _) in group_registry.definitions() {
                current_alias_names.insert(name.clone());
            }
            let mut current_defs_norm: Vec<(String, AHashSet<String>)> = Vec::new();
            for (name, def) in group_registry.definitions() {
                let mut set: AHashSet<String> = AHashSet::default();
                for u in &def.utilities {
                    if u.is_empty() {
                        continue;
                    }
                    if u.contains('@') {
                        continue;
                    }
                    if current_alias_names.contains(u) {
                        continue;
                    }
                    if group_registry.is_internal_token(u) {
                        continue;
                    }
                    set.insert(u.clone());
                }
                if !set.is_empty() {
                    current_defs_norm.push((name.clone(), set));
                }
            }

            let mut html_string = String::from_utf8_lossy(&html_bytes).to_string();
            let mut modified = false;
            for (old_name, old_def) in prev_registry.definitions() {
                if group_registry
                    .definitions()
                    .find(|(n, _)| *n == old_name)
                    .is_some()
                {
                    continue; // still exists with same name
                }
                let mut prev_set: AHashSet<String> = AHashSet::default();
                for u in &old_def.utilities {
                    if u.is_empty() {
                        continue;
                    }
                    if u.contains('@') {
                        continue;
                    }
                    if current_alias_names.contains(u) {
                        continue;
                    }
                    if group_registry.is_internal_token(u) {
                        continue;
                    }
                    prev_set.insert(u.clone());
                }
                if prev_set.is_empty() {
                    continue;
                }

                // Find best fuzzy match by Jaccard similarity
                let mut best_score = 0f64;
                let mut best_alias: Option<String> = None;
                for (cand_alias, cand_set) in &current_defs_norm {
                    let inter = prev_set.iter().filter(|x| cand_set.contains(*x)).count();
                    let uni = prev_set.len() + cand_set.len() - inter;
                    if uni == 0 {
                        continue;
                    }
                    let score = (inter as f64) / (uni as f64);
                    if score > best_score {
                        best_score = score;
                        best_alias = Some(cand_alias.clone());
                    }
                }
                let threshold: f64 = std::env::var("DX_GROUP_RENAME_SIMILARITY")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.6);
                let allow_aggressive_env =
                    std::env::var("DX_GROUP_AGGRESSIVE_REWRITE").ok().as_deref() == Some("1");
                if let Some(new_name) = best_alias {
                    if (best_score >= threshold && new_name.as_str() != old_name.as_str())
                        || allow_aggressive_env
                    {
                        if std::env::var("DX_DEBUG").ok().as_deref() == Some("1") {
                            eprintln!(
                                "[dx-style-debug] candidate rename: {} -> {} score={} threshold={} allow_aggressive={}",
                                old_name, new_name, best_score, threshold, allow_aggressive_env
                            );
                        }
                        let old_with_paren = format!("@{}(", old_name);
                        let new_with_paren = format!("@{}(", new_name);
                        if html_string.contains(&old_with_paren) {
                            html_string = html_string.replace(&old_with_paren, &new_with_paren);
                            modified = true;
                        }
                        let old_at = format!("@{}", old_name);
                        let new_at = format!("@{}", new_name);
                        if html_string.contains(&old_at) {
                            html_string = html_string.replace(&old_at, &new_at);
                            modified = true;
                        }

                        // Plain-token rename: replace class tokens exactly equal to the
                        // old alias name with @new_name. This handles `class="bg"`
                        // and `class="bg flex"` scenarios so they migrate to the
                        // grouped alias form.
                        let class_attr_re =
                            regex::Regex::new(r#"class\s*=\s*\"([^\"]*)\""#).unwrap();
                        let mut plain_html = html_string.clone();
                        let mut plain_modified = false;
                        for cap in class_attr_re.captures_iter(&html_string) {
                            if let Some(m) = cap.get(0) {
                                let full = m.as_str();
                                if let Some(group) = cap.get(1) {
                                    let classes_str = group.as_str();
                                    let mut items: Vec<String> = classes_str
                                        .split_whitespace()
                                        .map(|s| s.to_string())
                                        .collect();
                                    let mut replaced_any = false;
                                    for it in items.iter_mut() {
                                        if it == old_name {
                                            // Migrate plain token to plain alias (no leading '@')
                                            *it = new_name.clone();
                                            replaced_any = true;
                                        }
                                    }
                                    if replaced_any {
                                        let new_attr = format!("class=\"{}\"", items.join(" "));
                                        plain_html = plain_html.replacen(full, &new_attr, 1);
                                        plain_modified = true;
                                    }
                                }
                            }
                        }
                        if plain_modified {
                            html_string = plain_html;
                            modified = true;
                        }

                        // Aggressive rewrite: replace plain utility tokens with the
                        // alias when the class attribute appears to contain several
                        // utilities from the previous alias. This uses a simple
                        // class-attribute-aware replacement with a configurable
                        // overlap threshold to avoid false positives.
                        let overlap_threshold: f64 =
                            std::env::var("DX_GROUP_REWRITE_UTILITY_OVERLAP")
                                .ok()
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0.5);
                        // Build prev_set (normalized) for quick checks
                        let mut prev_set: AHashSet<String> = AHashSet::default();
                        if let Some(prev_def) =
                            prev_registry.definitions().find(|(n, _)| *n == old_name)
                        {
                            for u in &prev_def.1.utilities {
                                if u.is_empty() {
                                    continue;
                                }
                                if u.contains('@') {
                                    continue;
                                }
                                if group_registry.is_internal_token(u) {
                                    continue;
                                }
                                prev_set.insert(u.clone());
                            }
                        }
                        if !prev_set.is_empty() {
                            if std::env::var("DX_DEBUG").ok().as_deref() == Some("1") {
                                eprintln!(
                                    "[dx-style-debug] prev_set for '{}' = {:?}",
                                    old_name, prev_set
                                );
                            }
                            // Regex to find class="..." attributes
                            let class_attr_re =
                                regex::Regex::new(r#"class\s*=\s*\"([^\"]*)\""#).unwrap();
                            let mut new_html = html_string.clone();
                            for cap in class_attr_re.captures_iter(&html_string) {
                                if let Some(m) = cap.get(0) {
                                    let full = m.as_str();
                                    if let Some(group) = cap.get(1) {
                                        let classes_str = group.as_str();
                                        let items: Vec<&str> =
                                            classes_str.split_whitespace().collect();
                                        let total = items.len();
                                        if total == 0 {
                                            continue;
                                        }
                                        let mut match_count = 0usize;
                                        for it in &items {
                                            if prev_set.contains(&it.to_string()) {
                                                match_count += 1;
                                            }
                                        }
                                        let overlap = (match_count as f64) / (total as f64);
                                        if overlap >= overlap_threshold && match_count > 0 {
                                            if std::env::var("DX_DEBUG").ok().as_deref()
                                                == Some("1")
                                            {
                                                eprintln!(
                                                    "[dx-style-debug] class attr '{}' total={} match_count={} overlap={} -> will replace",
                                                    classes_str, total, match_count, overlap
                                                );
                                            }
                                            // Replace only the matching tokens with the new alias
                                            let mut replaced = false;
                                            let mut out_items: Vec<String> = Vec::new();
                                            for it in items {
                                                if prev_set.contains(&it.to_string()) {
                                                    // Use plain alias token (no leading '@')
                                                    let alias_token = new_name.clone();
                                                    if !out_items.contains(&alias_token) {
                                                        out_items.push(alias_token.clone());
                                                        replaced = true;
                                                    }
                                                } else {
                                                    out_items.push(it.to_string());
                                                }
                                            }
                                            if replaced {
                                                let new_classes = out_items.join(" ");
                                                let new_attr = format!("class=\"{}\"", new_classes);
                                                new_html = new_html.replacen(full, &new_attr, 1);
                                            }
                                        }
                                    }
                                }
                            }
                            if new_html != html_string {
                                html_string = new_html;
                                modified = true;
                            }
                        }
                    }
                }
            }
            if modified {
                std::fs::write(index_path, &html_string)?;
                html_bytes = html_string.into_bytes();
                // Re-extract and re-analyze with rewritten HTML
                let extracted2 =
                    extract_classes_fast(&html_bytes, prev_len_hint.next_power_of_two());
                let mut all_classes2 = extracted2.classes;
                group_registry = group::GroupRegistry::analyze(
                    &extracted2.group_events,
                    &mut all_classes2,
                    Some(AppState::engine()),
                );
                // If rewrite removed grouped alias metadata (manual toggle off), preserve previous entries
                let prev_registry2 = { state.lock().unwrap().group_registry.clone() };
                if prev_registry2.is_empty() == false && group_registry.is_empty() {
                    group_registry.merge_preserve(&prev_registry2);
                }
                // update master class set
                all_classes = all_classes2;
            }
        }
    }
    // If the rewrite removed grouped alias metadata (manual toggle off), but
    // our previous state had group definitions, merge those so we preserve
    // the cached CSS and definitions for developer convenience.
    {
        let prev_registry = { state.lock().unwrap().group_registry.clone() };
        if prev_registry.is_empty() == false && group_registry.is_empty() {
            group_registry.merge_preserve(&prev_registry);
        }
    }
    // If the HTML contains an explicit '@alias' token (even without
    // parentheses) treat that as a developer request to emit the legacy
    // dev selector. Use the group's recorded dev_tokens to synthesize a
    // `@alias(a b c)` string. This handles cases where the user writes
    // `class="@alias"` to request the dev selector be present.
    {
        // Optional pass: treat plain alias-name tokens (e.g. class="bg")
        // as a request to use the grouped alias by converting them to
        // `@alias`. This is gated behind DX_GROUP_REWRITE_PLAIN_ALIAS=1
        // so it won't run unless explicitly enabled.
        if std::env::var("DX_GROUP_REWRITE_PLAIN_ALIAS")
            .ok()
            .as_deref()
            == Some("1")
        {
            let class_attr_re = regex::Regex::new(r#"class\s*=\s*\"([^\"]*)\""#).unwrap();
            let html_string = String::from_utf8_lossy(&html_bytes).to_string();
            let mut new_html = html_string.clone();
            let mut any_mod = false;
            for (name, _def) in group_registry.definitions() {
                for cap in class_attr_re.captures_iter(&html_string) {
                    if let Some(m) = cap.get(0) {
                        let full = m.as_str();
                        if let Some(group) = cap.get(1) {
                            let classes_str = group.as_str();
                            let mut items: Vec<String> = classes_str
                                .split_whitespace()
                                .map(|s| s.to_string())
                                .collect();
                            let mut replaced = false;
                            for it in items.iter_mut() {
                                if it == name {
                                    *it = format!("@{}", name);
                                    replaced = true;
                                }
                            }
                            if replaced {
                                let new_attr = format!("class=\"{}\"", items.join(" "));
                                new_html = new_html.replacen(full, &new_attr, 1);
                                any_mod = true;
                            }
                        }
                    }
                }
            }
            if any_mod {
                std::fs::write(index_path, &new_html)?;
                html_bytes = new_html.into_bytes();
                // Re-extract and re-analyze with rewritten HTML
                let extracted2 =
                    extract_classes_fast(&html_bytes, all_classes.len().next_power_of_two());
                let mut all_classes2 = extracted2.classes;
                group_registry = group::GroupRegistry::analyze(
                    &extracted2.group_events,
                    &mut all_classes2,
                    Some(AppState::engine()),
                );
                // preserve previous registry entries if needed
                let prev_registry = { state.lock().unwrap().group_registry.clone() };
                if prev_registry.is_empty() == false && group_registry.is_empty() {
                    group_registry.merge_preserve(&prev_registry);
                }
                // update master class set
                all_classes = all_classes2;
            }
        }

        let mut devs = dev_group_selectors;
        // Inspect the HTML bytes for '@alias' occurrences. We purposely use
        // a simple substring search here to prefer a small, local change;
        // it's sufficient for typical usages like `@bg` or `@card`.
        let html_text = String::from_utf8_lossy(&html_bytes).to_string();
        for (name, def) in group_registry.definitions() {
            if devs.contains_key(name) {
                continue;
            }
            let search = format!("@{}", name);
            if html_text.contains(&search) {
                // Use dev_tokens when available, otherwise fall back to utilities
                let inner = if !def.dev_tokens.is_empty() {
                    def.dev_tokens.join(" ")
                } else {
                    def.utilities.join(" ")
                };
                if !inner.is_empty() {
                    devs.insert(name.clone(), format!("@{}({})", name, inner));
                }
            }
        }
        group_registry.set_dev_selectors(devs);
    }

    // Expand bare `@alias` occurrences in the HTML into `@alias(token1 token2)`
    // using the group's recorded dev_tokens or utilities. This ensures that
    // writing `class="@bg"` expands into the grouped form the generator
    // expects and yields the concrete utility tokens for CSS generation.
    {
        let mut html_string = String::from_utf8_lossy(&html_bytes).to_string();
        let mut modified = false;
        for (name, def) in group_registry.definitions() {
            let needle = format!("@{}", name);
            let mut start_idx = 0usize;
            while let Some(pos_rel) = html_string[start_idx..].find(&needle) {
                let pos = start_idx + pos_rel;
                let after = pos + needle.len();
                // If already followed by '(' then skip
                if html_string.as_bytes().get(after).map(|b| *b as char) == Some('(') {
                    start_idx = after;
                    continue;
                }
                let inner = if !def.dev_tokens.is_empty() {
                    def.dev_tokens.join(" ")
                } else {
                    def.utilities.join(" ")
                };
                if inner.is_empty() {
                    start_idx = after;
                    continue;
                }
                html_string.insert_str(after, &format!("({})", inner));
                modified = true;
                start_idx = after + inner.len() + 2;
            }
        }
        if modified {
            std::fs::write(index_path, &html_string)?;
            html_bytes = html_string.into_bytes();
            // Re-extract and re-analyze with expanded HTML
            let extracted2 = extract_classes_fast(&html_bytes, prev_len_hint.next_power_of_two());
            let mut all_classes2 = extracted2.classes;
            group_registry = group::GroupRegistry::analyze(
                &extracted2.group_events,
                &mut all_classes2,
                Some(AppState::engine()),
            );
            // preserve previous registry entries if needed
            let prev_registry = { state.lock().unwrap().group_registry.clone() };
            if prev_registry.is_empty() == false && group_registry.is_empty() {
                group_registry.merge_preserve(&prev_registry);
            }
            // Remove utility members now that we've re-analyzed
            group_registry.remove_utility_members_from(&mut all_classes2);
            all_classes = all_classes2;
        }
    }

    // Optional aggressive rewrite pass (env opt-in): examine class attributes
    // and replace plain utility tokens with the best matching @alias when the
    // overlap threshold is met. This is independent of previous registry state
    // and useful when migrating existing HTML to use grouped aliases.
    {
        let aggressive_env =
            std::env::var("DX_GROUP_AGGRESSIVE_REWRITE").ok().as_deref() == Some("1");
        if aggressive_env {
            // Build normalized utility sets for current groups
            let mut group_sets: Vec<(String, AHashSet<String>)> = Vec::new();
            let mut alias_names: AHashSet<String> = AHashSet::default();
            for (name, def) in group_registry.definitions() {
                alias_names.insert(name.clone());
                let mut set: AHashSet<String> = AHashSet::default();
                for u in &def.utilities {
                    if u.is_empty() {
                        continue;
                    }
                    if u.contains('@') {
                        continue;
                    }
                    if alias_names.contains(u) {
                        continue;
                    }
                    if group_registry.is_internal_token(u) {
                        continue;
                    }
                    set.insert(u.clone());
                }
                if !set.is_empty() {
                    group_sets.push((name.clone(), set));
                }
            }

            if !group_sets.is_empty() {
                let overlap_threshold: f64 = std::env::var("DX_GROUP_REWRITE_UTILITY_OVERLAP")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.5);
                let class_attr_re = regex::Regex::new(r#"class\s*=\s*\"([^\"]*)\""#).unwrap();
                let html_string = String::from_utf8_lossy(&html_bytes).to_string();
                // Snapshot of the HTML we started with (before aggressive pass)
                let original_html_string = String::from_utf8_lossy(&html_bytes).to_string();
                let mut new_html = html_string.clone();
                let mut any_mod = false;
                for cap in class_attr_re.captures_iter(&html_string) {
                    if let Some(m) = cap.get(0) {
                        let full = m.as_str();
                        // If the original HTML attribute already contained an explicit
                        // grouped alias (an '@'), don't touch it. This prevents the
                        // aggressive pass from stomping existing @alias usage.
                        if original_html_string.contains(full) {
                            if original_html_string
                                .find(full)
                                .and_then(|i| {
                                    original_html_string
                                        .get(i..i + full.len())
                                        .map(|s| s.contains('@'))
                                })
                                .unwrap_or(false)
                            {
                                // skip this attribute; it already had an '@' in source
                                continue;
                            }
                        }
                        if let Some(group) = cap.get(1) {
                            let classes_str = group.as_str();
                            let items: Vec<&str> = classes_str.split_whitespace().collect();
                            let total = items.len();
                            if total == 0 {
                                continue;
                            }
                            // Find best group by overlap
                            let mut best_score = 0f64;
                            let mut best_alias: Option<String> = None;
                            let mut best_match_count = 0usize;
                            for (alias, set) in &group_sets {
                                let match_count = items
                                    .iter()
                                    .filter(|it| set.contains(&it.to_string()))
                                    .count();
                                let score = (match_count as f64) / (total as f64);
                                if score > best_score {
                                    best_score = score;
                                    best_alias = Some(alias.clone());
                                    best_match_count = match_count;
                                }
                            }
                            if let Some(alias) = best_alias {
                                if best_score >= overlap_threshold && best_match_count > 0 {
                                    // Replace matching tokens with @alias
                                    let mut out_items: Vec<String> = Vec::new();
                                    let alias_token = format!("@{}", alias);
                                    let mut replaced = false;
                                    for it in items {
                                        if group_sets
                                            .iter()
                                            .any(|(_, s)| s.contains(&it.to_string()))
                                        {
                                            // only include alias once (plain alias)
                                            if !out_items.contains(&alias_token) {
                                                out_items.push(alias_token.clone());
                                                replaced = true;
                                            }
                                        } else {
                                            out_items.push(it.to_string());
                                        }
                                    }
                                    if replaced {
                                        let new_attr = format!("class=\"{}\"", out_items.join(" "));
                                        new_html = new_html.replacen(full, &new_attr, 1);
                                        any_mod = true;
                                        if std::env::var("DX_DEBUG").ok().as_deref() == Some("1") {
                                            eprintln!(
                                                "[dx-style-debug] aggressive replaced '{}' -> {}",
                                                full, new_attr
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if any_mod {
                    std::fs::write(index_path, &new_html)?;
                    html_bytes = new_html.into_bytes();
                    // Re-extract and re-analyze with rewritten HTML
                    let extracted2 =
                        extract_classes_fast(&html_bytes, all_classes.len().next_power_of_two());
                    let mut all_classes2 = extracted2.classes;
                    group_registry = group::GroupRegistry::analyze(
                        &extracted2.group_events,
                        &mut all_classes2,
                        Some(AppState::engine()),
                    );
                    // preserve previous registry entries if needed
                    let prev_registry = { state.lock().unwrap().group_registry.clone() };
                    if prev_registry.is_empty() == false && group_registry.is_empty() {
                        group_registry.merge_preserve(&prev_registry);
                    }
                    // Remove utility members now that we've re-analyzed
                    group_registry.remove_utility_members_from(&mut all_classes2);
                    all_classes = all_classes2;
                }
            }
        }
    }

    // Remove concrete utility members from the master class set so they are
    // neither persisted in the cache nor emitted as independent rules. The
    // grouped alias will provide the combined selector and bodies instead.
    // Before removing utility members, ensure we keep at most one grouped
    // controller per alias in the HTML. If multiple elements contain the
    // grouped form `@alias(...)`, convert all but one to the plain alias
    // token (e.g. `class="alias"` or `class="alias other"`). Prefer to
    // keep the grouped form on an element that already has extra tokens
    // (e.g. `@alias(...) flex`) so that the controller remains where the
    // element has additional behavior.
    {
        let mut html_string = String::from_utf8_lossy(&html_bytes).to_string();
        let class_attr_re = regex::Regex::new(r#"class\s*=\s*\"([^\"]*)\""#).unwrap();
        let mut modified = false;
        // Collect alias names
        let alias_names: Vec<String> = group_registry
            .definitions()
            .map(|(n, _)| n.clone())
            .collect();
        for alias in alias_names {
            // Find all class attributes that contain @alias(...)
            let grouped_re =
                regex::Regex::new(&format!(r"@{}\s*\(\s*([^\)]*)\s*\)", regex::escape(&alias)))
                    .unwrap();
            let mut occurrences: Vec<(String, String)> = Vec::new(); // (full_attr, classes_str)
            for cap in class_attr_re.captures_iter(&html_string) {
                if let Some(m) = cap.get(0) {
                    let full = m.as_str().to_string();
                    if let Some(group) = cap.get(1) {
                        let classes_str = group.as_str().to_string();
                        if grouped_re.is_match(&classes_str) {
                            occurrences.push((full, classes_str));
                        }
                    }
                }
            }
            if occurrences.len() <= 1 {
                continue;
            }
            // Choose keeper: prefer attribute that has other tokens besides the grouped token
            let mut keeper_idx: Option<usize> = None;
            for (i, (_full, classes_str)) in occurrences.iter().enumerate() {
                // count tokens that are not the grouped token
                let tokens: Vec<&str> = classes_str.split_whitespace().collect();
                let mut other_count = 0usize;
                for tk in &tokens {
                    if grouped_re.is_match(tk) {
                        continue;
                    }
                    other_count += 1;
                }
                if other_count > 0 {
                    keeper_idx = Some(i);
                    break;
                }
            }
            if keeper_idx.is_none() {
                keeper_idx = Some(0);
            }
            // Convert other occurrences to plain alias tokens inside the class attribute
            let mut new_html = html_string.clone();
            for (i, (full, classes_str)) in occurrences.iter().enumerate() {
                if Some(i) == keeper_idx {
                    continue;
                }
                // Replace only the grouped token(s) for this alias inside classes_str with the plain alias
                let new_classes = grouped_re.replace_all(classes_str, alias.as_str());
                let new_attr = format!("class=\"{}\"", new_classes);
                new_html = new_html.replacen(full, &new_attr, 1);
                modified = true;
            }
            if modified {
                html_string = new_html;
                // update occurrences for next alias (and persist changes)
            }
        }
        if modified {
            std::fs::write(index_path, &html_string)?;
            html_bytes = html_string.into_bytes();
            // Re-extract and re-analyze with rewritten HTML
            let extracted2 =
                extract_classes_fast(&html_bytes, all_classes.len().next_power_of_two());
            let mut all_classes2 = extracted2.classes;
            group_registry = group::GroupRegistry::analyze(
                &extracted2.group_events,
                &mut all_classes2,
                Some(AppState::engine()),
            );
            all_classes = all_classes2;
        }
    }
    group_registry.remove_utility_members_from(&mut all_classes);

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
                    // println!("{}", message);
                }
                state_guard.group_log_hash = new_hash;
            }
        }
    }
    let cache_update_duration = cache_update_timer.elapsed();

    // Persist cache including group dump when available. Lock once to extract
    // required pieces and then call save_cache with an optional group dump
    // reference to avoid holding the lock while performing IO.
    {
        let guard = state.lock().unwrap();
        let class_cache_copy = guard.class_cache.clone();
        let groups_opt = if guard.group_registry.is_empty() {
            None
        } else {
            Some(guard.group_registry.to_dump())
        };
        drop(guard);
        if let Err(e) = cache::save_cache(&class_cache_copy, new_html_hash, groups_opt.as_ref()) {
            eprintln!("{} {}", "Error saving cache:".red(), e);
        }
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
