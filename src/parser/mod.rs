use ahash::AHashSet;
use memchr::{memchr, memmem::Finder};

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
fn expand_grouping_into(s: &str, out: &mut AHashSet<String>) {
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

    use smallvec::SmallVec;
    let bytes = s.as_bytes();
    let n = bytes.len();
    let mut i = 0usize;
    let mut stack: SmallVec<[&str; 4]> = SmallVec::new();
    let mut tok_start: Option<usize> = None;

    #[inline]
    fn trim_plus(s: &str) -> &str {
        let mut end = s.len();
        let b = s.as_bytes();
        while end > 0 && b[end - 1] == b'+' {
            end -= 1;
        }
        &s[..end]
    }

    let finalize_token =
        |start: usize, end: usize, out: &mut AHashSet<String>, stack: &SmallVec<[&str; 4]>| {
            if end <= start {
                return;
            }
            let mut token = &s[start..end];
            token = trim_plus(token);
            if token.is_empty() {
                return;
            }
            if stack.is_empty() {
                out.insert(token.to_string());
            } else {
                let total_len = stack.iter().map(|p| p.len() + 1).sum::<usize>() + token.len();
                let mut combined = String::with_capacity(total_len);
                for (idx, p) in stack.iter().enumerate() {
                    if idx > 0 {
                        combined.push(':');
                    }
                    combined.push_str(p);
                }
                combined.push(':');
                combined.push_str(token);
                out.insert(combined);
            }
        };

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
                finalize_token(ts, i, out, &stack);
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
                let mut prefix = &s[ts..i];
                prefix = trim_plus(prefix);
                if !prefix.is_empty() {
                    stack.push(prefix);
                }
            }
            i += 1; // consume '('
            continue;
        }

        // Otherwise, finalize a normal token
        if let Some(ts) = tok_start.take() {
            finalize_token(ts, i, out, &stack);
        }
        // If current char is ')', the top of loop will pop it
    }

    // Final token at end-of-string
    if let Some(ts) = tok_start.take() {
        finalize_token(ts, n, out, &stack);
    }
}

pub fn extract_classes_fast(html_bytes: &[u8], capacity_hint: usize) -> AHashSet<String> {
    let mut set = AHashSet::with_capacity(capacity_hint.max(64));
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
            expand_grouping_into(value_str, &mut set);
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
            expand_grouping_into(value_str, &mut set);
        }
        pos = value_end + 1;
    }

    set
}
