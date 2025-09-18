use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};

// Format CSS prettily using lightningcss; ensure blank line separation after rules.
pub fn format_css_pretty(input: &str) -> Option<String> {
    let sheet = StyleSheet::parse(
        input,
        ParserOptions {
            error_recovery: true,
            ..ParserOptions::default()
        },
    )
    .ok()?;
    let printed = sheet
        .to_css(PrinterOptions {
            minify: false,
            ..PrinterOptions::default()
        })
        .ok()?;
    let mut out = printed.code;
    // Heuristic: ensure a blank line between adjacent class rules within @layer utilities
    // Replace single newline after closing brace followed directly by a class selector with double newline.
    out = out.replace("}\n.", "}\n\n.");
    Some(out)
}

// Quick function to detect if buffer contains parse errors by trying a strict parse.
// (removed unused has_parse_errors to silence warnings)
