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
fn fast_split_whitespace_insert(s: &str, out: &mut AHashSet<String>) {
    for cls in s.split_whitespace() {
        if !cls.is_empty() {
            out.insert(cls.to_owned());
        }
    }
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
                    if !trimmed.is_empty() {
                        let combined = if stack.is_empty() {
                            trimmed.to_string()
                        } else {
                            let total_len =
                                stack.iter().map(|p| p.len() + 1).sum::<usize>() + trimmed.len();
                            let mut combined = String::with_capacity(total_len);
                            for (idx, p) in stack.iter().enumerate() {
                                if idx > 0 {
                                    combined.push(':');
                                }
                                combined.push_str(p);
                            }
                            combined.push(':');
                            combined.push_str(trimmed);
                            combined
                        };
                        out.insert(combined.clone());
                        collector.record(stack.as_slice(), trimmed, had_plus, &combined);
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
                    if !trimmed.is_empty() {
                        stack.push(trimmed.to_string());
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
                if !trimmed.is_empty() {
                    let combined = if stack.is_empty() {
                        trimmed.to_string()
                    } else {
                        let total_len =
                            stack.iter().map(|p| p.len() + 1).sum::<usize>() + trimmed.len();
                        let mut combined = String::with_capacity(total_len);
                        for (idx, p) in stack.iter().enumerate() {
                            if idx > 0 {
                                combined.push(':');
                            }
                            combined.push_str(p);
                        }
                        combined.push(':');
                        combined.push_str(trimmed);
                        combined
                    };
                    out.insert(combined.clone());
                    collector.record(stack.as_slice(), trimmed, had_plus, &combined);
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
            if !trimmed.is_empty() {
                let combined = if stack.is_empty() {
                    trimmed.to_string()
                } else {
                    let total_len =
                        stack.iter().map(|p| p.len() + 1).sum::<usize>() + trimmed.len();
                    let mut combined = String::with_capacity(total_len);
                    for (idx, p) in stack.iter().enumerate() {
                        if idx > 0 {
                            combined.push(':');
                        }
                        combined.push_str(p);
                    }
                    combined.push(':');
                    combined.push_str(trimmed);
                    combined
                };
                out.insert(combined.clone());
                collector.record(stack.as_slice(), trimmed, had_plus, &combined);
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
            occurrences.push(ClassOccurrence {
                attr_range: attr_start..attr_end,
                tokens,
                canonical,
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
    let mut alias_counter: usize = 1;

    for indices in grouped.values() {
        if indices.len() < 2 {
            continue;
        }
        let first_idx = indices[0];
        let tokens = occurrences[first_idx].tokens.clone();
        let alias = loop {
            let candidate = format!("dxg-{}", alias_counter);
            alias_counter += 1;
            if !existing_names.contains(&candidate) {
                existing_names.insert(candidate.clone());
                break candidate;
            }
        };
        let tokens_join = tokens.join(" ");
        let first_range = occurrences[first_idx].attr_range.clone();
        replacements.push((
            first_range,
            format!(
                "class=\"{}\" dx-group=\"{}({})\"",
                alias, alias, tokens_join
            ),
        ));
        for &idx in &indices[1..] {
            let range = occurrences[idx].attr_range.clone();
            replacements.push((range, format!("class=\"{}\"", alias)));
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
        let html = br#"<h1 class="border flex text-red-500">Hello</h1>
<h1 class="border flex text-red-500">World</h1>"#;
        let result = rewrite_duplicate_classes(html).expect("rewrite");
        let rewritten = String::from_utf8(result.html.clone()).unwrap();
        assert!(
            rewritten.contains("class=\"dxg-1\" dx-group=\"dxg-1(border flex text-red-500)\""),
            "expected alias definition"
        );
        assert!(
            rewritten.contains("class=\"dxg-1\">World"),
            "expected second occurrence to use alias"
        );
        assert_eq!(result.groups.len(), 1);
        assert_eq!(result.groups[0].alias, "dxg-1");
        assert_eq!(
            result.groups[0].classes,
            vec![
                "border".to_string(),
                "flex".to_string(),
                "text-red-500".to_string()
            ]
        );
    }
}
