use ahash::AHashSet;
use memchr::{memchr, memmem::Finder};

// Split a whitespace-separated class list while preserving whitespace inside balanced parentheses.
fn split_preserving_groups(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut depth: i32 = 0;
    for ch in s.chars() {
        match ch {
            '(' => {
                depth += 1;
                cur.push(ch);
            }
            ')' => {
                if depth > 0 {
                    depth -= 1;
                }
                cur.push(ch);
            }
            ' ' | '\n' | '\r' | '\t' if depth == 0 => {
                if !cur.trim().is_empty() {
                    out.push(cur.trim().to_string());
                }
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

fn is_reserved_func_prefix(p: &str) -> bool {
    matches!(p, "from" | "to" | "via")
}

// Recursively expands grouping like prefix(arg1 arg2 ...) into [prefix:arg1, prefix:arg2, ...].
fn expand_with_prefix(prefix: &str, token: &str) -> Vec<String> {
    // If token itself is a grouping, decompose and recurse
    if let Some(open) = token.find('(') {
        // Ensure closing ')' at end and balanced
        if token.ends_with(')') {
            let head = token[..open].trim();
            if head.is_empty() {
                return vec![format!(
                    "{}{}{}",
                    if prefix.is_empty() { "" } else { prefix },
                    if prefix.is_empty() { "" } else { ":" },
                    token
                )];
            }
            // Avoid expanding reserved animation-like tokens or external refs
            if head.starts_with('@') || is_reserved_func_prefix(head) {
                return vec![format!(
                    "{}{}{}",
                    if prefix.is_empty() { "" } else { prefix },
                    if prefix.is_empty() { "" } else { ":" },
                    token
                )];
            }
            // Check balance inside
            let inner = &token[open + 1..token.len() - 1];
            let mut bal = 0i32;
            for c in inner.chars() {
                if c == '(' {
                    bal += 1;
                }
                if c == ')' {
                    bal -= 1;
                    if bal < 0 {
                        return vec![token.to_string()];
                    }
                }
            }
            // Combine prefix with head
            let combined = if prefix.is_empty() {
                head.to_string()
            } else {
                format!("{}:{}", prefix, head)
            };
            let mut out = Vec::new();
            for part in split_preserving_groups(inner) {
                for leaf in expand_with_prefix(&combined, &part) {
                    out.push(leaf);
                }
            }
            return out;
        }
    }
    if prefix.is_empty() {
        vec![token.to_string()]
    } else {
        vec![format!("{}:{}", prefix, token)]
    }
}

fn maybe_expand_group_token(token: &str) -> Option<Vec<String>> {
    if let Some(open) = token.find('(') {
        if token.ends_with(')') {
            let head = token[..open].trim();
            if head.is_empty() || head.starts_with('@') || is_reserved_func_prefix(head) {
                return None;
            }
            // simple balance check
            let mut bal = 0i32;
            for c in token[open + 1..token.len() - 1].chars() {
                if c == '(' {
                    bal += 1;
                }
                if c == ')' {
                    bal -= 1;
                    if bal < 0 {
                        return None;
                    }
                }
            }
            return Some(expand_with_prefix("", token));
        }
    }
    None
}

fn normalize_leaf(token: &str) -> Option<String> {
    let t = token.trim();
    if t.is_empty() {
        return None;
    }
    let t = if let Some(stripped) = t.strip_suffix('+') {
        stripped.trim_end()
    } else {
        t
    };
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

fn parse_attr_value_at(html: &[u8], name_start: usize) -> Option<(usize, usize)> {
    // Parse attribute name [a-zA-Z0-9_-]+ possibly with preceding whitespace already checked
    let n = html.len();
    let mut i = name_start;
    while i < n {
        let b = html[i];
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' {
            i += 1;
        } else {
            break;
        }
    }
    // Skip whitespace to '='
    while i < n && matches!(html[i], b' ' | b'\n' | b'\r' | b'\t') {
        i += 1;
    }
    if i >= n || html[i] != b'=' {
        return None;
    }
    i += 1;
    while i < n && matches!(html[i], b' ' | b'\n' | b'\r' | b'\t') {
        i += 1;
    }
    if i >= n {
        return None;
    }
    let quote = html[i];
    if quote != b'"' && quote != b'\'' {
        return None;
    }
    let value_start = i + 1;
    if let Some(rel) = memchr(quote, &html[value_start..]) {
        let value_end = value_start + rel;
        Some((value_start, value_end))
    } else {
        None
    }
}

pub fn extract_classes_fast(html_bytes: &[u8], capacity_hint: usize) -> AHashSet<String> {
    let mut set = AHashSet::with_capacity(capacity_hint.max(64));
    let n = html_bytes.len();

    // 1) Scan standard class attributes quickly (as before) but with grouping-aware splitting
    let class_finder = Finder::new(b"class");
    let mut pos = 0usize;
    while let Some(idx) = class_finder.find(&html_bytes[pos..]) {
        let start = pos + idx + 5;
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
        if let Ok(mut value_str) = std::str::from_utf8(&html_bytes[value_start..value_end]) {
            if let Some(hash) = value_str.find('#') {
                value_str = &value_str[..hash];
            }
            for raw in split_preserving_groups(value_str) {
                if raw.is_empty() {
                    continue;
                }
                if let Some(expanded) = maybe_expand_group_token(&raw) {
                    for e in expanded {
                        if let Some(n) = normalize_leaf(&e) {
                            set.insert(n);
                        }
                    }
                } else {
                    if let Some(n) = normalize_leaf(&raw) {
                        set.insert(n);
                    }
                }
            }
        }
        pos = value_end + 1;
    }

    // 2) Also scan for dx-* attributes and parse similarly.
    let dx_finder = Finder::new(b"dx-");
    let mut pos2 = 0usize;
    while let Some(idx) = dx_finder.find(&html_bytes[pos2..]) {
        let name_start = pos2 + idx;
        if let Some((vstart, vend)) = parse_attr_value_at(html_bytes, name_start) {
            if let Ok(mut value_str) = std::str::from_utf8(&html_bytes[vstart..vend]) {
                if let Some(hash) = value_str.find('#') {
                    value_str = &value_str[..hash];
                }
                for raw in split_preserving_groups(value_str) {
                    if raw.is_empty() {
                        continue;
                    }
                    if let Some(expanded) = maybe_expand_group_token(&raw) {
                        for e in expanded {
                            if let Some(n) = normalize_leaf(&e) {
                                set.insert(n);
                            }
                        }
                    } else {
                        if let Some(n) = normalize_leaf(&raw) {
                            set.insert(n);
                        }
                    }
                }
            }
            pos2 = vend + 1;
        } else {
            pos2 = name_start + 3;
        }
    }

    set
}
