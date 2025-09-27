use ahash::{AHashMap, AHashSet};
use memchr::{memchr, memmem::Finder};
use smallvec::SmallVec;
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct GroupEvent {
    pub stack: Vec<String>,
    pub token: String,
    pub had_plus: bool,
    pub full_class: String,
}

#[derive(Debug, Default, Clone)]
pub struct GroupCollector {
    events: Vec<GroupEvent>,
}

impl GroupCollector {
    #[inline]
    fn record(&mut self, stack: &[String], token: &str, had_plus: bool, full_class: &str) {
        if stack.is_empty() {
            return;
        }
        self.events.push(GroupEvent {
            stack: stack.iter().cloned().collect(),
            token: token.to_string(),
            had_plus,
            full_class: full_class.to_string(),
        });
    }

    pub fn into_events(self) -> Vec<GroupEvent> {
        self.events
    }
}

pub struct ExtractedClasses {
    pub classes: AHashSet<String>,
    pub group_events: Vec<GroupEvent>,
}

#[derive(Debug, Clone)]
pub struct AutoGroupInfo {
    pub alias: String,
    pub classes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AutoGroupRewrite {
    pub html: Vec<u8>,
    pub groups: Vec<AutoGroupInfo>,
}

#[inline]
fn base_alias_from_tokens(tokens: &[String]) -> String {
    let mut alias = String::new();
    for token in tokens {
        if alias.len() >= 5 {
            break;
        }
        if let Some(ch) = token.chars().find(|c| c.is_ascii_alphabetic()) {
            alias.push(ch.to_ascii_lowercase());
        }
    }
    if alias.is_empty() {
        for token in tokens {
            for ch in token.chars() {
                if ch.is_ascii_alphanumeric() {
                    alias.push(ch.to_ascii_lowercase());
                    if alias.len() >= 5 {
                        break;
                    }
                }
            }
            if alias.len() >= 5 {
                break;
            }
        }
    }
    if alias.is_empty() {
        alias.push_str("grp");
    }
    if alias.len() > 5 {
        alias.truncate(5);
    }
    alias
}

#[inline]
fn fast_split_whitespace_insert(s: &str, out: &mut AHashSet<String>) {
    for cls in s.split_whitespace() {
        if !cls.is_empty() {
            out.insert(cls.to_owned());
        }
    }
}

#[inline]
fn sanitize_group_token(raw: &str) -> &str {
    if let Some(rest) = raw.strip_prefix('@') {
        if !rest.is_empty() {
            return rest;
        }
    }
    raw
}

// Expand grouping syntax like: lg(bg-red-500 text-yellow-500) -> lg:bg-red-500, lg:text-yellow-500
// Supports nesting: lg(md(bg-red-500)) -> lg:md:bg-red-500
// Ignores inline comments starting with '#' and trailing '+' suffixes on tokens.
#[inline]
fn expand_grouping_into(s: &str, out: &mut AHashSet<String>, collector: &mut GroupCollector) {
    // Cut off inline comment starting with '#'
    let s = match s.as_bytes().iter().position(|&b| b == b'#') {
        Some(i) => &s[..i],
        None => s,
    };
    // Fast path: no grouping/comment/plus symbols -> simple split
    if !s
        .as_bytes()
        .iter()
        .any(|&b| matches!(b, b'(' | b')' | b'+'))
    {
        fast_split_whitespace_insert(s, out);
        return;
    }

    let bytes = s.as_bytes();
    let n = bytes.len();
    let mut i = 0usize;
    let mut stack: SmallVec<[String; 4]> = SmallVec::new();
    let mut tok_start: Option<usize> = None;

    #[inline]
    fn trim_plus(s: &str) -> (&str, bool) {
        let mut end = s.len();
        let b = s.as_bytes();
        let mut had_plus = false;
        while end > 0 && b[end - 1] == b'+' {
            end -= 1;
            had_plus = true;
        }
        (&s[..end], had_plus)
    }

    while i < n {
        // Skip whitespace
        while i < n && matches!(bytes[i], b' ' | b'\n' | b'\r' | b'\t') {
            i += 1;
        }
        if i >= n {
            break;
        }

        // Handle closing parens possibly repeated
        while i < n && bytes[i] == b')' {
            if let Some(ts) = tok_start.take() {
                if ts < i {
                    let raw = &s[ts..i];
                    let (trimmed, had_plus) = trim_plus(raw);
                    let sanitized = sanitize_group_token(trimmed);
                    if !sanitized.is_empty() {
                        let combined = if stack.is_empty() {
                            sanitized.to_string()
                        } else {
                            let total_len =
                                stack.iter().map(|p| p.len() + 1).sum::<usize>() + sanitized.len();
                            let mut combined = String::with_capacity(total_len);
                            for (idx, p) in stack.iter().enumerate() {
                                if idx > 0 {
                                    combined.push(':');
                                }
                                combined.push_str(p);
                            }
                            combined.push(':');
                            combined.push_str(sanitized);
                            combined
                        };
                        out.insert(combined.clone());
                        collector.record(stack.as_slice(), sanitized, had_plus, &combined);
                    }
                }
            }
            if !stack.is_empty() {
                stack.pop();
            }
            i += 1;
            // skip whitespace after )
            while i < n && matches!(bytes[i], b' ' | b'\n' | b'\r' | b'\t') {
                i += 1;
            }
        }
        if i >= n {
            break;
        }

        // Start a token if not in one
        if tok_start.is_none() {
            tok_start = Some(i);
        }

        // Advance until whitespace or paren
        while i < n && !matches!(bytes[i], b' ' | b'\n' | b'\r' | b'\t' | b'(' | b')') {
            i += 1;
        }

        // If next is '(', treat previous token as a prefix and push
        if i < n && bytes[i] == b'(' {
            if let Some(ts) = tok_start.take() {
                if ts < i {
                    let raw = &s[ts..i];
                    let (trimmed, _) = trim_plus(raw);
                    let sanitized = sanitize_group_token(trimmed);
                    if !sanitized.is_empty() {
                        stack.push(sanitized.to_string());
                    }
                }
            }
            i += 1; // consume '('
            continue;
        }

        // Otherwise, finalize a normal token
        if let Some(ts) = tok_start.take() {
            if ts < i {
                let raw = &s[ts..i];
                let (trimmed, had_plus) = trim_plus(raw);
                let sanitized = sanitize_group_token(trimmed);
                if !sanitized.is_empty() {
                    let combined = if stack.is_empty() {
                        sanitized.to_string()
                    } else {
                        let total_len =
                            stack.iter().map(|p| p.len() + 1).sum::<usize>() + sanitized.len();
                        let mut combined = String::with_capacity(total_len);
                        for (idx, p) in stack.iter().enumerate() {
                            if idx > 0 {
                                combined.push(':');
                            }
                            combined.push_str(p);
                        }
                        combined.push(':');
                        combined.push_str(sanitized);
                        combined
                    };
                    out.insert(combined.clone());
                    collector.record(stack.as_slice(), sanitized, had_plus, &combined);
                }
            }
        }
        // If current char is ')', the top of loop will pop it
    }

    // Final token at end-of-string
    if let Some(ts) = tok_start.take() {
        if ts < n {
            let raw = &s[ts..n];
            let (trimmed, had_plus) = trim_plus(raw);
            let sanitized = sanitize_group_token(trimmed);
            if !sanitized.is_empty() {
                let combined = if stack.is_empty() {
                    sanitized.to_string()
                } else {
                    let total_len =
                        stack.iter().map(|p| p.len() + 1).sum::<usize>() + sanitized.len();
                    let mut combined = String::with_capacity(total_len);
                    for (idx, p) in stack.iter().enumerate() {
                        if idx > 0 {
                            combined.push(':');
                        }
                        combined.push_str(p);
                    }
                    combined.push(':');
                    combined.push_str(sanitized);
                    combined
                };
                out.insert(combined.clone());
                collector.record(stack.as_slice(), sanitized, had_plus, &combined);
            }
        }
    }
}

pub fn extract_classes_fast(html_bytes: &[u8], capacity_hint: usize) -> ExtractedClasses {
    let mut set = AHashSet::with_capacity(capacity_hint.max(64));
    let mut collector = GroupCollector::default();
    let mut pos = 0usize;
    let n = html_bytes.len();

    // Pass 1: standard class="..."
    let class_finder = Finder::new(b"class");
    while let Some(idx) = class_finder.find(&html_bytes[pos..]) {
        let start = pos + idx + 5; // after 'class'
        let mut i = start;
        while i < n && matches!(html_bytes[i], b' ' | b'\n' | b'\r' | b'\t') {
            i += 1;
        }
        if i >= n || html_bytes[i] != b'=' {
            pos = start;
            continue;
        }
        i += 1;
        while i < n && matches!(html_bytes[i], b' ' | b'\n' | b'\r' | b'\t') {
            i += 1;
        }
        if i >= n {
            break;
        }
        let quote = html_bytes[i];
        if quote != b'"' && quote != b'\'' {
            pos = i;
            continue;
        }
        i += 1;
        let value_start = i;
        let rel_end = memchr(quote, &html_bytes[value_start..]);
        let value_end = match rel_end {
            Some(off) => value_start + off,
            None => break,
        };
        if let Ok(value_str) = std::str::from_utf8(&html_bytes[value_start..value_end]) {
            // Use grouping-aware extractor for both, but it fast-paths when absent
            expand_grouping_into(value_str, &mut set, &mut collector);
        }
        pos = value_end + 1;
    }

    // Pass 2: dx-*="..." attributes
    pos = 0;
    let dx_finder = Finder::new(b"dx-");
    while let Some(idx) = dx_finder.find(&html_bytes[pos..]) {
        let mut i = pos + idx; // at 'd' in 'dx-'
        // read attribute name [a-zA-Z0-9_-]+
        i += 3; // skip 'dx-'
        while i < n {
            let b = html_bytes[i];
            if (b as char).is_ascii_alphanumeric() || b == b'-' || b == b'_' {
                i += 1;
            } else {
                break;
            }
        }
        // skip whitespace
        while i < n && matches!(html_bytes[i], b' ' | b'\n' | b'\r' | b'\t') {
            i += 1;
        }
        if i >= n || html_bytes[i] != b'=' {
            pos = pos + idx + 3;
            continue;
        }
        i += 1;
        while i < n && matches!(html_bytes[i], b' ' | b'\n' | b'\r' | b'\t') {
            i += 1;
        }
        if i >= n {
            break;
        }
        let quote = html_bytes[i];
        if quote != b'"' && quote != b'\'' {
            pos = pos + idx + 3;
            continue;
        }
        i += 1;
        let value_start = i;
        let rel_end = memchr(quote, &html_bytes[value_start..]);
        let value_end = match rel_end {
            Some(off) => value_start + off,
            None => break,
        };
        if let Ok(value_str) = std::str::from_utf8(&html_bytes[value_start..value_end]) {
            expand_grouping_into(value_str, &mut set, &mut collector);
        }
        pos = value_end + 1;
    }

    ExtractedClasses {
        classes: set,
        group_events: collector.into_events(),
    }
}

#[derive(Debug)]
struct ClassOccurrence {
    attr_range: Range<usize>,
    tokens: Vec<String>,
    canonical: String,
    dx_group_cleanup: Option<(Range<usize>, String)>,
}

pub fn rewrite_duplicate_classes(html_bytes: &[u8]) -> Option<AutoGroupRewrite> {
    let mut occurrences: Vec<ClassOccurrence> = Vec::new();
    let mut existing_names: AHashSet<String> = AHashSet::default();

    let mut pos = 0usize;
    let n = html_bytes.len();
    let finder = Finder::new(b"class");
    while let Some(idx) = finder.find(&html_bytes[pos..]) {
        let attr_start = pos + idx;
        if attr_start > 0 {
            let prev = html_bytes[attr_start - 1];
            if (prev as char).is_ascii_alphanumeric() || prev == b'-' || prev == b'_' {
                pos = attr_start + 5;
                continue;
            }
        }
        let mut cursor = attr_start + 5;
        while cursor < n && matches!(html_bytes[cursor], b' ' | b'\n' | b'\r' | b'\t') {
            cursor += 1;
        }
        if cursor >= n || html_bytes[cursor] != b'=' {
            pos = attr_start + 5;
            continue;
        }
        cursor += 1;
        while cursor < n && matches!(html_bytes[cursor], b' ' | b'\n' | b'\r' | b'\t') {
            cursor += 1;
        }
        if cursor >= n {
            break;
        }
        let quote = html_bytes[cursor];
        if quote != b'"' && quote != b'\'' {
            pos = attr_start + 5;
            continue;
        }
        cursor += 1;
        let value_start = cursor;
        let rel_end = memchr(quote, &html_bytes[value_start..]);
        let value_end = match rel_end {
            Some(off) => value_start + off,
            None => break,
        };
        let attr_end = value_end + 1;
        if let Ok(value_str) = std::str::from_utf8(&html_bytes[value_start..value_end]) {
            let raw_tokens: Vec<&str> = value_str
                .split_whitespace()
                .filter(|t| !t.is_empty())
                .collect();
            for tok in &raw_tokens {
                existing_names.insert((*tok).to_string());
            }
            if raw_tokens.len() < 2 {
                pos = attr_end;
                continue;
            }
            if value_str.as_bytes().iter().any(|b| {
                matches!(
                    *b,
                    b'(' | b')' | b'{' | b'}' | b':' | b'@' | b'#' | b'[' | b']'
                )
            }) {
                pos = attr_end;
                continue;
            }
            let mut seen: AHashSet<&str> = AHashSet::with_capacity(raw_tokens.len());
            let mut simple = true;
            for &tok in &raw_tokens {
                if tok.contains('+') || tok.starts_with("dxg-") {
                    simple = false;
                    break;
                }
                if !seen.insert(tok) {
                    simple = false;
                    break;
                }
            }
            if !simple {
                pos = attr_end;
                continue;
            }
            let tokens: Vec<String> = raw_tokens.iter().map(|t| (*t).to_string()).collect();
            let canonical = tokens.join("\u{0}");

            let whitespace_start = attr_end;
            let mut attr_ws_end = whitespace_start;
            while attr_ws_end < n && matches!(html_bytes[attr_ws_end], b' ' | b'\n' | b'\r' | b'\t')
            {
                attr_ws_end += 1;
            }
            let mut dx_group_cleanup: Option<(Range<usize>, String)> = None;
            if attr_ws_end + 8 <= n && &html_bytes[attr_ws_end..attr_ws_end + 8] == b"dx-group" {
                let mut cursor_after_name = attr_ws_end + 8;
                while cursor_after_name < n
                    && matches!(html_bytes[cursor_after_name], b' ' | b'\n' | b'\r' | b'\t')
                {
                    cursor_after_name += 1;
                }
                if cursor_after_name < n && html_bytes[cursor_after_name] == b'=' {
                    cursor_after_name += 1;
                    while cursor_after_name < n
                        && matches!(html_bytes[cursor_after_name], b' ' | b'\n' | b'\r' | b'\t')
                    {
                        cursor_after_name += 1;
                    }
                    if cursor_after_name < n {
                        let dx_quote = html_bytes[cursor_after_name];
                        if dx_quote == b'"' || dx_quote == b'\'' {
                            cursor_after_name += 1;
                            let value_start = cursor_after_name;
                            if let Some(off) = memchr(dx_quote, &html_bytes[value_start..]) {
                                let value_end = value_start + off;
                                cursor_after_name = value_end + 1;
                                let mut trailing = cursor_after_name;
                                while trailing < n
                                    && matches!(html_bytes[trailing], b' ' | b'\n' | b'\r' | b'\t')
                                {
                                    trailing += 1;
                                }
                                let has_following_attr =
                                    trailing < n && !matches!(html_bytes[trailing], b'>' | b'/');
                                let prefix_bytes = &html_bytes[whitespace_start..attr_ws_end];
                                let prefix =
                                    std::str::from_utf8(prefix_bytes).unwrap_or(" ").to_string();
                                let replacement = if has_following_attr {
                                    if prefix.is_empty() {
                                        " ".to_string()
                                    } else {
                                        prefix
                                    }
                                } else {
                                    String::new()
                                };
                                dx_group_cleanup = Some((whitespace_start..trailing, replacement));
                            }
                        }
                    }
                }
            }
            occurrences.push(ClassOccurrence {
                attr_range: attr_start..attr_end,
                tokens,
                canonical,
                dx_group_cleanup,
            });
        }
        pos = attr_end;
    }

    if occurrences.is_empty() {
        return None;
    }

    let mut grouped: AHashMap<String, Vec<usize>> = AHashMap::default();
    for (idx, occ) in occurrences.iter().enumerate() {
        grouped.entry(occ.canonical.clone()).or_default().push(idx);
    }

    let mut replacements: Vec<(Range<usize>, String)> = Vec::new();
    let mut infos: Vec<AutoGroupInfo> = Vec::new();
    let mut alias_counts: AHashMap<String, usize> = AHashMap::default();

    for indices in grouped.values() {
        if indices.len() < 2 {
            continue;
        }
        let first_idx = indices[0];
        let tokens = occurrences[first_idx].tokens.clone();
        let base_alias = base_alias_from_tokens(&tokens);
        let mut suffix = alias_counts.get(&base_alias).copied().unwrap_or(0);
        let mut candidate = if suffix == 0 {
            base_alias.clone()
        } else {
            format!("{}{}", base_alias, suffix)
        };
        while existing_names.contains(&candidate) {
            suffix += 1;
            candidate = format!("{}{}", base_alias, suffix);
        }
        alias_counts.insert(base_alias.clone(), suffix + 1);
        existing_names.insert(candidate.clone());
        let alias = candidate;
        let tokens_join = tokens.join(" ");
        let first_range = occurrences[first_idx].attr_range.clone();
        replacements.push((
            first_range,
            format!("class=\"{} @{}({})\"", alias, alias, tokens_join),
        ));
        if let Some((range, replacement)) = occurrences[first_idx].dx_group_cleanup.clone() {
            replacements.push((range, replacement));
        }
        for &idx in &indices[1..] {
            let range = occurrences[idx].attr_range.clone();
            replacements.push((range, format!("class=\"{}\"", alias)));
            if let Some((range, replacement)) = occurrences[idx].dx_group_cleanup.clone() {
                replacements.push((range, replacement));
            }
        }
        infos.push(AutoGroupInfo {
            alias,
            classes: tokens,
        });
    }

    if replacements.is_empty() {
        return None;
    }

    replacements.sort_by(|a, b| b.0.start.cmp(&a.0.start));
    let mut html_string = String::from_utf8(html_bytes.to_vec()).ok()?;
    for (range, replacement) in replacements {
        html_string.replace_range(range, &replacement);
    }

    Some(AutoGroupRewrite {
        html: html_string.into_bytes(),
        groups: infos,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouping_extracts_alias_and_utilities() {
        let html = br#"<div dx-text="card(bg-red-500 h-50 text-yellow-500+)"></div>"#;
        let extracted = extract_classes_fast(html, 0);
        assert!(
            !extracted.classes.contains("bg-red-500"),
            "base utility should be restored during analysis"
        );
        assert!(
            !extracted.classes.contains("h-50"),
            "base utility should be restored during analysis"
        );
        assert!(
            !extracted.classes.contains("text-yellow-500"),
            "base utility should be restored during analysis"
        );
        assert!(
            !extracted.classes.contains("card"),
            "alias should be added during analysis, not raw extraction"
        );
        assert!(extracted.classes.contains("card:bg-red-500"));
        let alias_event = extracted
            .group_events
            .iter()
            .find(|evt| evt.stack == vec!["card".to_string()] && evt.token == "text-yellow-500");
        assert!(alias_event.is_some(), "expected event for card group");
        assert!(
            alias_event.unwrap().had_plus,
            "plus suffix should be recorded"
        );

        let mut classes = extracted.classes.clone();
        let registry =
            crate::core::group::GroupRegistry::analyze(&extracted.group_events, &mut classes, None);
        assert!(
            classes.contains("card"),
            "alias should be registered after analysis"
        );
        assert!(classes.contains("bg-red-500"));
        assert!(classes.contains("h-50"));
        assert!(classes.contains("text-yellow-500"));
        assert!(
            !classes.contains("card:bg-red-500"),
            "internal grouped tokens should be dropped after analysis"
        );
        assert!(
            registry.definitions().any(|(name, _)| name == "card"),
            "registry should capture alias definition"
        );
    }

    #[test]
    fn rewrite_duplicates_into_group_alias() {
        let html = br#"<h1 class="border flex text-red-500" dx-group="old(alias)">Hello</h1>
<h1 class="border flex text-red-500">World</h1>"#;
        let result = rewrite_duplicate_classes(html).expect("rewrite");
        let rewritten = String::from_utf8(result.html.clone()).unwrap();
        assert!(
            rewritten.contains("class=\"bft @bft(border flex text-red-500)\""),
            "expected alias definition to use class-only grouping"
        );
        assert!(
            rewritten.contains("class=\"bft\">World"),
            "expected second occurrence to use alias only"
        );
        assert!(
            !rewritten.contains("dx-group"),
            "legacy dx-group attribute should be stripped"
        );
        assert_eq!(result.groups.len(), 1);
        assert_eq!(result.groups[0].alias, "bft");
        assert_eq!(
            result.groups[0].classes,
            vec![
                "border".to_string(),
                "flex".to_string(),
                "text-red-500".to_string()
            ]
        );
    }

    #[test]
    fn rewrite_alias_avoids_existing_names() {
        let html = br#"<div class="bft">Existing</div>
<div class="border flex text-red-500">Hello</div>
<div class="border flex text-red-500">World</div>"#;
        let result = rewrite_duplicate_classes(html).expect("rewrite");
        let rewritten = String::from_utf8(result.html.clone()).unwrap();
        assert!(
            rewritten.contains("class=\"bft\">Existing"),
            "untouched class should remain"
        );
        assert!(
            rewritten.contains("class=\"bft1 @bft1(border flex text-red-500)\""),
            "expected alias to avoid collision via numeric suffix"
        );
        assert!(
            rewritten.contains("class=\"bft1\">World"),
            "expected subsequent match to use the alias"
        );
        assert_eq!(result.groups.len(), 1);
        assert_eq!(result.groups[0].alias, "bft1");
    }

    #[test]
    fn rewrite_alias_truncates_to_five_letters() {
        let html = br#"<div class="border flex uppercase text-red-500 after shadow">One</div>
<div class="border flex uppercase text-red-500 after shadow">Two</div>"#;
        let result = rewrite_duplicate_classes(html).expect("rewrite");
        let rewritten = String::from_utf8(result.html.clone()).unwrap();
        assert!(
            rewritten.contains(
                "class=\"bfuta @bfuta(border flex uppercase text-red-500 after shadow)\""
            ),
            "expected alias to use first five initials"
        );
        assert!(
            result.groups.iter().any(|g| g.alias == "bfuta"),
            "alias metadata should record truncated name"
        );
    }
}
