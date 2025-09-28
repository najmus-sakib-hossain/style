use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};

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
    out = out.replace("}\n.", "}\n\n.");
    Some(out)
}
