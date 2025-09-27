use ahash::{AHashMap, AHashSet};

use crate::parser::GroupEvent;

use super::engine::StyleEngine;

#[derive(Debug, Default, Clone)]
pub struct GroupDefinition {
    pub utilities: Vec<String>,
    pub allow_extend: bool,
    pub raw_tokens: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct GroupRegistry {
    definitions: AHashMap<String, GroupDefinition>,
    internal_tokens: AHashSet<String>,
    cached_css: AHashMap<String, String>,
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

    pub fn is_internal_token(&self, class: &str) -> bool {
        self.internal_tokens.contains(class)
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
                });
            if !entry.utilities.contains(&actual_class) {
                entry.utilities.push(actual_class.clone());
            }
            entry.raw_tokens.push(event.full_class.clone());
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

        registry
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

        let mut accumulated = String::new();
        let alias_selector = make_selector(class);
        for util in flattened {
            if let Some(mut css) = engine.css_for_class(&util) {
                rewrite_selector(&mut css, &util, &alias_selector);
                accumulated.push_str(&css);
                if !accumulated.ends_with('\n') {
                    accumulated.push('\n');
                }
            }
        }
        if accumulated.is_empty() {
            return None;
        }

        let entry = self
            .cached_css
            .entry(class.to_string())
            .or_insert_with(|| accumulated);
        Some(entry.as_str())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::extract_classes_fast;
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
        let css = registry.generate_css_for("card", &engine).expect("css");
        assert!(css.contains(".card"));
        assert!(css.contains("background-color: red"));
        assert!(css.contains("height"));
        let _ = std::fs::remove_file(&temp_path);
    }
}
