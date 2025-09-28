use ahash::{AHashMap, AHashSet};

use crate::parser::GroupEvent;

use super::engine::StyleEngine;

#[derive(Debug, Default, Clone)]
pub struct GroupDefinition {
    pub utilities: Vec<String>,
    pub allow_extend: bool,
    pub raw_tokens: Vec<String>,
    pub dev_tokens: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct GroupRegistry {
    definitions: AHashMap<String, GroupDefinition>,
    internal_tokens: AHashSet<String>,
    utility_members: AHashSet<String>,
    cached_css: AHashMap<String, String>,
    dev_selectors: AHashMap<String, String>,
}

impl GroupRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    pub fn definitions(&self) -> impl Iterator<Item = (&String, &GroupDefinition)> {
        self.definitions.iter()
    }

    pub fn set_dev_selectors(&mut self, selectors: AHashMap<String, String>) {
        // Set dev selectors exactly as provided by the caller. This allows the caller
        // (e.g. the auto-group rewrite) to control whether legacy `@alias(...)` dev
        // selectors are present. Do not auto-insert defaults here to support toggling.
        if !selectors.is_empty() {
            // println!(
            //     "[dx-style] set_dev_selectors -> {}",
            //     selectors.keys().cloned().collect::<Vec<_>>().join(",")
            // );
        }
        // Replace dev selectors exactly as provided. Previously this cleared
        // the entire cached_css which forced regeneration / lost historical
        // alias rules when only dev selectors changed. Instead, only remove
        // cached entries for aliases that no longer exist in the new
        // selector map (they might have been intentionally removed).
        let old_selectors = std::mem::replace(&mut self.dev_selectors, selectors);

        // If there was no previous selector map, keep cache as-is.
        if old_selectors.is_empty() {
            return;
        }

        // Remove cached_css for aliases that were removed from dev selectors
        // to avoid keeping dev-only derived rules around. Do not clear other
        // cached entries so manual edits to dev selectors don't force users
        // to update old class names in HTML/CSS.
        let removed: Vec<String> = old_selectors
            .keys()
            .filter(|k| !self.dev_selectors.contains_key(*k))
            .cloned()
            .collect();
        for alias in removed {
            self.cached_css.remove(&alias);
        }
    }

    pub fn is_internal_token(&self, class: &str) -> bool {
        self.internal_tokens.contains(class)
    }

    /// Merge definitions and cached CSS from a previous registry, preserving
    /// any entries that don't exist in self. This is used when the parser
    /// removes grouped utilities from the HTML (manual toggle) but we want to
    /// keep the previously-generated CSS for those aliases in the output.
    pub fn merge_preserve(&mut self, prev: &GroupRegistry) {
        // First, copy any missing definitions from previous registry.
        for (name, def) in prev.definitions.iter() {
            self.definitions
                .entry(name.clone())
                .or_insert_with(|| def.clone());
        }

        // Build alias name set for normalization to exclude alias tokens
        // from utility comparisons.
        let mut current_alias_names: AHashSet<String> = AHashSet::default();
        for (name, _) in self.definitions.iter() {
            current_alias_names.insert(name.clone());
        }

        // Build a vector of (alias, normalized_util_set) for current defs to
        // support fuzzy matching (Jaccard) when migrating aliases.
        let mut current_defs_norm: Vec<(String, AHashSet<String>)> = Vec::new();
        for (name, def) in self.definitions.iter() {
            let mut set: AHashSet<String> = AHashSet::default();
            for u in &def.utilities {
                if u.is_empty() {
                    continue;
                }
                if u.contains('@') {
                    continue;
                }
                if current_alias_names.contains(u) {
                    // u is itself an alias name; exclude
                    continue;
                }
                set.insert(u.clone());
            }
            if !set.is_empty() {
                current_defs_norm.push((name.clone(), set));
            }
        }
        // Only preserve cached CSS entries when the utilities for the alias
        // are identical between the previous registry and the current one.
        // This avoids keeping stale CSS for aliases whose underlying
        // utilities changed (which would produce incorrect output).
        for (k, v) in prev.cached_css.iter() {
            // If we already have a definition for this alias, check utilities
            // equality. If the utilities differ, skip preserving the cached CSS
            // so it can be regenerated.
            if let Some(current_def) = self.definitions.get(k) {
                if let Some(prev_def) = prev.definitions.get(k) {
                    if current_def.utilities == prev_def.utilities {
                        self.cached_css
                            .entry(k.clone())
                            .or_insert_with(|| v.clone());
                    } else {
                        // utilities changed -> do not preserve cached CSS
                    }
                    continue;
                }
            }

            // If alias key not present in current definitions, attempt to
            // find a renamed alias with an identical utilities set and
            // migrate the cached CSS under the new alias name.
            if let Some(prev_def) = prev.definitions.get(k) {
                // Normalize prev utilities same as we did for current defs
                let mut prev_set: AHashSet<String> = AHashSet::default();
                for u in &prev_def.utilities {
                    if u.is_empty() {
                        continue;
                    }
                    if u.contains('@') {
                        continue;
                    }
                    // also exclude any current alias names to avoid matching
                    // nested aliases
                    if current_alias_names.contains(u) {
                        continue;
                    }
                    prev_set.insert(u.clone());
                }
                if !prev_set.is_empty() {
                    // Find best fuzzy match by Jaccard similarity
                    let mut best_score = 0f64;
                    let mut best_alias: Option<String> = None;
                    for (cand_alias, cand_set) in &current_defs_norm {
                        // compute intersection/union
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
                    // Similarity threshold: allow fuzzy match; configurable via env
                    let threshold: f64 = std::env::var("DX_GROUP_RENAME_SIMILARITY")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.6);
                    if let Some(new_alias) = best_alias {
                        if best_score >= threshold {
                            // Insert under new alias if missing
                            self.cached_css
                                .entry(new_alias.clone())
                                .or_insert_with(|| v.clone());
                            continue;
                        }
                    }
                }
            }

            // Otherwise preserve the cached CSS under the old key so older
            // aliases remain available if nothing else matches.
            self.cached_css
                .entry(k.clone())
                .or_insert_with(|| v.clone());
        }
        for tok in prev.utility_members.iter() {
            self.utility_members.insert(tok.clone());
        }
    }

    /// Create a serializable dump of definitions and cached CSS for persistence.
    pub fn to_dump(&self) -> super::super::cache::GroupDump {
        use std::collections::BTreeMap;
        let mut defs = BTreeMap::new();
        for (k, v) in self.definitions.iter() {
            defs.insert(
                k.clone(),
                super::super::cache::GroupDefDump {
                    utilities: v.utilities.clone(),
                    allow_extend: v.allow_extend,
                    raw_tokens: v.raw_tokens.clone(),
                    dev_tokens: v.dev_tokens.clone(),
                },
            );
        }
        let mut css_map = BTreeMap::new();
        for (k, v) in self.cached_css.iter() {
            css_map.insert(k.clone(), v.clone());
        }
        super::super::cache::GroupDump {
            definitions: defs,
            cached_css: css_map,
        }
    }

    /// Restore registry state from a dump (used when loading cache).
    pub fn from_dump(dump: &super::super::cache::GroupDump) -> Self {
        let mut registry = GroupRegistry::new();
        for (k, v) in dump.definitions.iter() {
            registry.definitions.insert(
                k.clone(),
                GroupDefinition {
                    utilities: v.utilities.clone(),
                    allow_extend: v.allow_extend,
                    raw_tokens: v.raw_tokens.clone(),
                    dev_tokens: v.dev_tokens.clone(),
                },
            );
            for util in &v.utilities {
                registry.utility_members.insert(util.clone());
            }
        }
        for (k, v) in dump.cached_css.iter() {
            registry.cached_css.insert(k.clone(), v.clone());
        }
        registry
    }

    pub fn analyze(
        events: &[GroupEvent],
        classes: &mut AHashSet<String>,
        engine: Option<&StyleEngine>,
    ) -> Self {
        let mut registry = GroupRegistry::default();
        if events.is_empty() {
            return registry;
        }

        let mut known_prefixes: AHashSet<String> = AHashSet::default();
        if let Some(engine) = engine {
            known_prefixes.extend(engine.screens.keys().cloned());
            known_prefixes.extend(engine.states.keys().cloned());
            known_prefixes.extend(engine.container_queries.keys().cloned());
        }

        for event in events {
            if event.stack.is_empty() {
                continue;
            }
            let mut alias_idx: Option<usize> = None;
            for (idx, seg) in event.stack.iter().enumerate() {
                if known_prefixes.contains(seg) {
                    continue;
                }
                alias_idx = Some(idx);
                break;
            }
            let Some(idx) = alias_idx else {
                continue;
            };
            let alias_name = event.stack[idx].clone();
            if alias_name.is_empty() {
                continue;
            }
            let recognized_prefixes = &event.stack[..idx];
            let actual_class = if recognized_prefixes.is_empty() {
                event.token.clone()
            } else {
                build_prefixed_class(recognized_prefixes, &event.token)
            };

            // Register alias definition
            let entry = registry
                .definitions
                .entry(alias_name.clone())
                .or_insert_with(|| GroupDefinition {
                    utilities: Vec::new(),
                    allow_extend: false,
                    raw_tokens: Vec::new(),
                    dev_tokens: Vec::new(),
                });
            if !entry.utilities.contains(&actual_class) {
                entry.utilities.push(actual_class.clone());
            }
            // Record that this concrete utility is a member of some group so
            // the generator can skip emitting the utility individually.
            registry.utility_members.insert(actual_class.clone());
            entry.raw_tokens.push(event.full_class.clone());
            if !entry.dev_tokens.contains(&event.token) {
                entry.dev_tokens.push(event.token.clone());
            }
            if event.had_plus {
                entry.allow_extend = true;
            }

            registry.internal_tokens.insert(event.full_class.clone());
            classes.insert(alias_name);
            classes.insert(actual_class);
        }

        for token in registry.internal_tokens.iter() {
            classes.remove(token);
        }

        // Note: do NOT remove utility members here. The caller (rebuild_styles)
        // will decide when to strip concrete utilities from the final class set
        // before emitting or persisting them. Tests that call analyze directly
        // expect the classes set to remain intact.

        registry
    }

    /// Remove any concrete utility members from the provided class set. This
    /// is intended to be called by the rebuild pipeline after analysis so that
    /// grouped utilities are not emitted or saved as standalone utilities.
    pub fn remove_utility_members_from(&self, classes: &mut AHashSet<String>) {
        for util in self.utility_members.iter() {
            classes.remove(util);
        }
    }

    pub fn is_util_member(&self, class: &str) -> bool {
        self.utility_members.contains(class)
    }

    pub fn generate_css_for<'a>(
        &'a mut self,
        class: &str,
        engine: &StyleEngine,
    ) -> Option<&'a str> {
        if self.internal_tokens.contains(class) {
            return None;
        }
        let utilities = match self.definitions.get(class) {
            Some(def) => def.utilities.clone(),
            None => return None,
        };

        let mut visited: AHashSet<String> = AHashSet::default();
        let mut flattened: Vec<String> = Vec::new();
        let mut seen: AHashSet<String> = AHashSet::default();
        for util in &utilities {
            collect_final_classes(self, util, &mut visited, &mut flattened, &mut seen);
        }

        if flattened.is_empty() {
            return None;
        }

        let alias_selector = make_selector(class);
        let dev_selector = self
            .dev_selectors
            .get(class)
            .and_then(|raw| parse_grouped_selector(raw, alias_selector.as_str()));
        let combined_selector = dev_selector
            .as_ref()
            .map(|dev| format!("{},{}", alias_selector, dev));
        let mut simple_bodies: Vec<String> = Vec::new();
        let mut extra_css = String::new();
        let mut missing_utils: Vec<String> = Vec::new();
        for util in flattened {
            if let Some(mut css) = engine.css_for_class(&util) {
                rewrite_selector(&mut css, &util, &alias_selector);
                let trimmed_css = css.trim();
                let mut handled_simple = false;
                if let Some(open_idx) = trimmed_css.find('{') {
                    if trimmed_css.ends_with('}') {
                        let selector = trimmed_css[..open_idx].trim();
                        let body = trimmed_css[open_idx + 1..trimmed_css.len() - 1].trim();
                        if selector == alias_selector
                            && !selector.contains(',')
                            && !body.contains('{')
                            && !body.contains('}')
                        {
                            simple_bodies.push(body.to_string());
                            handled_simple = true;
                        }
                    }
                }
                if !handled_simple {
                    if !extra_css.is_empty() && !extra_css.ends_with('\n') {
                        extra_css.push('\n');
                    }
                    extra_css.push_str(trimmed_css);
                    if !trimmed_css.ends_with('\n') {
                        extra_css.push('\n');
                    }
                }
            } else {
                // Record missing utility so we can log why a group might not emit.
                missing_utils.push(util.clone());
            }
        }

        let mut simple_block = String::new();
        if !simple_bodies.is_empty() {
            // Emit a rule for the alias selector. If a dev selector exists,
            // emit a combined selector to include both the alias and dev forms.
            let selector_output = combined_selector
                .as_deref()
                .unwrap_or_else(|| alias_selector.as_str());
            // Alias/combined rule
            simple_block.push_str(selector_output);
            simple_block.push_str(" {\n");
            for body in &simple_bodies {
                for line in body.lines() {
                    let trimmed_line = line.trim();
                    if trimmed_line.is_empty() {
                        continue;
                    }
                    simple_block.push_str("  ");
                    simple_block.push_str(trimmed_line);
                    if !trimmed_line.ends_with(';') && !trimmed_line.ends_with('}') {
                        simple_block.push(';');
                    }
                    simple_block.push('\n');
                }
            }
            simple_block.push_str("}\n");
        }

        let mut accumulated = String::new();
        if !simple_block.is_empty() {
            accumulated.push_str(&simple_block);
        }
        if !extra_css.is_empty() {
            if !accumulated.is_empty() && !accumulated.ends_with('\n') {
                accumulated.push('\n');
            }
            // Emit extra (complex) rules for both alias and dev selector when
            // a dev selector exists. For alias we output the original extra_css
            // (which references alias_selector). For dev we produce a copy with
            // alias_selector replaced by the dev selector so the same complex
            // rules apply to the @... form as well.
            accumulated.push_str(&extra_css);
            if let Some(ref dev_sel) = dev_selector {
                if !extra_css.ends_with('\n') {
                    accumulated.push('\n');
                }
                accumulated.push_str(&extra_css.replace(alias_selector.as_str(), dev_sel));
            }
        }

        if accumulated.trim().is_empty() {
            if !missing_utils.is_empty() {
                // println!(
                //     "[dx-style] group '{}' skipped: missing utils -> {}",
                //     class,
                //     missing_utils.join(" ")
                // );
            }
            return None;
        }

        // If we already have a cached value but the regenerated CSS differs,
        // replace it so changes to utilities or dev selectors are reflected
        // in output. Otherwise insert the newly-generated value.
        // Clone existing cached value (if any) so we don't hold an immutable
        // borrow while inserting. Compare the clone with the newly-generated
        // content and return early if unchanged. Otherwise replace the map
        // entry.
        let existing_clone = self.cached_css.get(class).map(|s| s.clone());
        if let Some(ref old) = existing_clone {
            if old == &accumulated {
                // Safe to return a reference from the map because we haven't
                // mutated it.
                return Some(self.cached_css.get(class).unwrap().as_str());
            }
        }

        self.cached_css.insert(class.to_string(), accumulated);
        Some(self.cached_css.get(class).unwrap().as_str())
    }
}

fn build_prefixed_class(prefixes: &[String], token: &str) -> String {
    if prefixes.is_empty() {
        return token.to_string();
    }
    let total_len = prefixes.iter().map(|p| p.len() + 1).sum::<usize>() + token.len();
    let mut out = String::with_capacity(total_len);
    for (idx, prefix) in prefixes.iter().enumerate() {
        if idx > 0 {
            out.push(':');
        }
        out.push_str(prefix);
    }
    out.push(':');
    out.push_str(token);
    out
}

fn collect_final_classes(
    registry: &GroupRegistry,
    util: &str,
    visited: &mut AHashSet<String>,
    out: &mut Vec<String>,
    seen: &mut AHashSet<String>,
) {
    if seen.contains(util) {
        return;
    }
    if visited.contains(util) {
        return;
    }
    if let Some(def) = registry.definitions.get(util) {
        visited.insert(util.to_string());
        for child in &def.utilities {
            collect_final_classes(registry, child, visited, out, seen);
        }
        visited.remove(util);
    } else {
        if seen.insert(util.to_string()) {
            out.push(util.to_string());
        }
    }
}

fn make_selector(class: &str) -> String {
    let mut escaped = String::new();
    cssparser::serialize_identifier(class, &mut escaped).unwrap();
    format!(".{}", escaped)
}

fn rewrite_selector(css: &mut String, original: &str, alias_selector: &str) {
    let mut escaped_original = String::new();
    cssparser::serialize_identifier(original, &mut escaped_original).unwrap();
    let original_selector = format!(".{}", escaped_original);
    if original_selector == alias_selector {
        return;
    }
    *css = css.replace(&original_selector, alias_selector);
}

fn parse_grouped_selector(raw: &str, alias_selector: &str) -> Option<String> {
    // TODO(dev-only): drop legacy @alias(...) selector once dev tools stop requiring it.
    let raw = raw.trim();
    if raw.is_empty() || !raw.starts_with('@') {
        return None;
    }
    let Some(open_idx) = raw.find('(') else {
        return None;
    };
    let alias_part = raw[1..open_idx].trim();
    if alias_part.is_empty() {
        return None;
    }
    let Some(inner) = raw[open_idx + 1..].trim().strip_suffix(')') else {
        return None;
    };

    let mut parsed_alias = String::new();
    cssparser::serialize_identifier(alias_part, &mut parsed_alias).ok()?;
    let expected_alias = alias_selector.strip_prefix('.')?;
    if parsed_alias != expected_alias {
        return None;
    }

    let mut inner_sanitized = String::new();
    let mut first = true;
    for token in inner.split_whitespace() {
        if token.is_empty() {
            continue;
        }
        if !first {
            inner_sanitized.push(' ');
        }
        first = false;
        let mut sanitized = String::new();
        cssparser::serialize_identifier(token, &mut sanitized).ok()?;
        inner_sanitized.push_str(&sanitized);
    }
    if inner_sanitized.is_empty() {
        return None;
    }

    let mut class_name = String::new();
    class_name.push('@');
    class_name.push_str(&parsed_alias);
    class_name.push('(');
    class_name.push_str(&inner_sanitized);
    class_name.push(')');

    let mut escaped = String::with_capacity(class_name.len() + 1);
    escaped.push('.');
    cssparser::serialize_identifier(&class_name, &mut escaped).ok()?;
    Some(escaped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::extract_classes_fast;
    use ahash::AHashMap;
    use std::io::Write;

    #[test]
    fn alias_generates_combined_css() {
        let html = br#"<div dx-text="card(bg-red-500 h-50)"></div>"#;
        let extracted = extract_classes_fast(html, 0);
        let mut classes = extracted.classes.clone();
        let temp_path = std::env::temp_dir().join("dx_style_test_style.bin");
        {
            let mut f = std::fs::File::create(&temp_path).unwrap();
            f.write_all(&vec![0u8; 4096]).unwrap();
        }
        unsafe {
            std::env::set_var("DX_STYLE_BIN", &temp_path);
        }
        let mut engine = crate::core::engine::StyleEngine::empty();
        unsafe {
            std::env::remove_var("DX_STYLE_BIN");
        }
        engine.precompiled.insert(
            "bg-red-500".to_string(),
            "background-color: red;".to_string(),
        );
        engine
            .precompiled
            .insert("h-50".to_string(), "height: 12.5rem;".to_string());
        let registry = GroupRegistry::analyze(&extracted.group_events, &mut classes, Some(&engine));
        assert!(classes.contains("card"));
        let mut registry = registry;
        let mut selectors = AHashMap::default();
        selectors.insert("card".to_string(), "@card(bg-red-500 h-50)".to_string());
        registry.set_dev_selectors(selectors);
        let css = registry.generate_css_for("card", &engine).expect("css");
        assert!(css.contains(".card"));
        assert!(css.contains(".card,.\\@card\\(bg-red-500\\ h-50\\)"));
        assert!(css.contains("background-color: red"));
        assert!(css.contains("height"));
        let _ = std::fs::remove_file(&temp_path);
    }
}
