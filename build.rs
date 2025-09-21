use flatbuffers::{FlatBufferBuilder, WIPOffset};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
struct GeneratorConfig {
    multiplier: f32,
    unit: String,
}

#[derive(Deserialize, Debug)]
struct StaticConfig {
    #[serde(rename = "static")]
    static_styles: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct DynamicConfig {
    dynamic: HashMap<String, HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
struct GeneratorsConfig {
    generators: HashMap<String, GeneratorConfig>,
}

#[derive(Deserialize, Debug)]
struct ScreensConfig {
    screens: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct StatesConfig {
    states: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct ContainerQueriesConfig {
    container_queries: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct ColorsConfig {
    colors: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct AnimationGeneratorsConfig {
    animation_generators: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct PropertyMetaConfig {
    syntax: String,
    #[serde(default)]
    inherits: Option<bool>,
    #[serde(default, rename = "initial")]
    initial_value: Option<String>,
}

#[derive(Deserialize, Debug)]
struct PropertiesConfig {
    properties: HashMap<String, PropertyMetaConfig>,
}

fn read_toml_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Option<T> {
    if path.exists() {
        let content = fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    } else {
        None
    }
}

fn read_file_string(path: &Path) -> Option<String> {
    if !path.exists() {
        println!("cargo:warning=TOML path not found: {}", path.display());
        return None;
    }
    match fs::read_to_string(path) {
        Ok(s) => Some(s),
        Err(e) => {
            println!(
                "cargo:warning=Failed to read file at {}: {}",
                path.display(),
                e
            );
            None
        }
    }
}

fn toml_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(n) = chars.next() {
                match n {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    _ => {
                        out.push(n);
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn css_properties_from_toml(path: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Some(content) = read_file_string(path) else {
        return map;
    };
    let mut current: Option<String> = None;
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            let inner = &line[1..line.len() - 1].trim();
            // Only accept simple top-level headers like ["display"] (no dot)
            if let Some(first_quote) = inner.find('"') {
                if let Some(second_quote) = inner[first_quote + 1..].find('"') {
                    let name = &inner[first_quote + 1..first_quote + 1 + second_quote];
                    if !inner.contains('.') {
                        current = Some(name.to_string());
                    } else {
                        current = None;
                    }
                } else {
                    current = None;
                }
            } else {
                current = None;
            }
            continue;
        }
        if let Some(ref name) = current {
            // Looking for a line like: "syntax" = "..."
            if let Some(eq_idx) = line.find('=') {
                let key = line[..eq_idx].trim();
                let val = line[eq_idx + 1..].trim();
                if key == "\"syntax\"" {
                    if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                        let raw = &val[1..val.len() - 1];
                        let syn = toml_unescape(raw);
                        map.insert(name.clone(), syn);
                    }
                }
            }
        }
    }
    map
}

fn last_segment(name: &str) -> &str {
    name.rsplit('-').next().unwrap_or(name)
}

fn letters_only(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_lowercase()
}

fn compute_property_abbreviations(names: &[String]) -> HashMap<String, String> {
    // First letter per property; if multiple share the first letter, use first-letter + '-' +
    // minimal unique prefix of the last hyphen-separated segment among that group.
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<char, Vec<String>> = BTreeMap::new();
    for n in names {
        let first = letters_only(n).chars().next().unwrap_or('x');
        groups.entry(first).or_default().push(n.clone());
    }
    let mut out: HashMap<String, String> = HashMap::new();
    for (first, list) in groups {
        if list.len() == 1 {
            out.insert(list[0].clone(), first.to_string());
            continue;
        }
        // Build last segments
        let lasts: Vec<String> = list
            .iter()
            .map(|name| {
                name.split('-')
                    .last()
                    .map(letters_only)
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| letters_only(name))
            })
            .collect();
        // Determine minimal unique prefix length for each last
        let mut prefixes: Vec<String> = vec![String::new(); list.len()];
        let mut plen = 1usize;
        loop {
            // Propose prefixes of length plen (or full last if shorter)
            let proposed: Vec<String> = lasts
                .iter()
                .map(|l| l.chars().take(plen).collect::<String>())
                .collect();
            // Count occurrences
            let mut counts: BTreeMap<String, usize> = BTreeMap::new();
            for p in &proposed {
                *counts.entry(p.clone()).or_insert(0) += 1;
            }
            // Check which are unique
            let mut all_unique = true;
            for (i, p) in proposed.iter().enumerate() {
                if counts[p] == 1 {
                    prefixes[i] = p.clone();
                } else {
                    all_unique = false;
                }
            }
            if all_unique {
                break;
            }
            plen += 1;
            if plen > 10 {
                // Fallback to full last segments
                for i in 0..prefixes.len() {
                    if prefixes[i].is_empty() {
                        prefixes[i] = lasts[i].clone();
                    }
                }
                break;
            }
        }
        for (i, name) in list.iter().enumerate() {
            let abbr = format!("{}-{}", first, prefixes[i].clone());
            out.insert(name.clone(), abbr);
        }
    }
    out
}

fn strip_angle_brackets(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut depth = 0i32;
    for c in s.chars() {
        match c {
            '<' => depth += 1,
            '>' => depth = (depth - 1).max(0),
            _ => {
                if depth == 0 {
                    out.push(c);
                }
            }
        }
    }
    out
}

fn extract_keywords_from_syntax(syntax: &str) -> Vec<String> {
    let cleaned = strip_angle_brackets(syntax);
    let mut words: Vec<String> = Vec::new();
    let mut cur = String::new();
    for c in cleaned.chars() {
        if c.is_ascii_alphabetic() || c == '-' {
            cur.push(c.to_ascii_lowercase());
        } else {
            if !cur.is_empty() {
                words.push(cur.clone());
                cur.clear();
            }
        }
    }
    if !cur.is_empty() {
        words.push(cur);
    }
    // Deduplicate while preserving order
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for w in words.into_iter().filter(|w| !w.is_empty()) {
        if seen.insert(w.clone()) {
            out.push(w);
        }
    }
    out
}

fn compute_value_abbreviations(values: &[String]) -> HashMap<String, String> {
    use std::cmp::min;
    let mut map = HashMap::new();
    if values.is_empty() {
        return map;
    }
    // Work on hyphen-stripped keys for prefix calc
    let cleaned: Vec<(String, String)> = values
        .iter()
        .map(|v| (v.clone(), letters_only(v)))
        .collect();
    let mut prefix_len = 1usize;
    loop {
        let mut seen: HashMap<String, Vec<String>> = HashMap::new();
        for (orig, clean) in &cleaned {
            let l = min(prefix_len, clean.len());
            let ab = clean[..l].to_string();
            seen.entry(ab).or_default().push(orig.clone());
        }
        let all_unique = seen.values().all(|v| v.len() == 1);
        if all_unique {
            for (abbr, v) in seen {
                map.insert(v[0].clone(), abbr);
            }
            break;
        }
        prefix_len += 1;
        if prefix_len > 10 {
            // Fallback: map to full cleaned string
            for (orig, clean) in &cleaned {
                map.insert(orig.clone(), clean.clone());
            }
            break;
        }
    }
    map
}

fn main() {
    let style_dir_str = (|| {
        if let Ok(content) = std::fs::read_to_string(".dx/config.toml") {
            if let Ok(value) = content.parse::<toml::Value>() {
                if let Some(path_val) = value.get("paths").and_then(|p| p.get("style_dir")) {
                    if let Some(s) = path_val.as_str() {
                        if !s.trim().is_empty() {
                            return s.replace('\\', "/");
                        }
                    }
                }
            }
        }
        ".dx/style".to_string()
    })();
    let style_dir = Path::new(&style_dir_str);
    let fbs_path = style_dir.join("style.fbs");
    let fbs_files_vec = vec![fbs_path.to_string_lossy().to_string()];
    let fbs_files: Vec<&Path> = fbs_files_vec.iter().map(|s| Path::new(s)).collect();
    let out_dir = std::env::var("OUT_DIR").unwrap();

    for fbs_file in &fbs_files_vec {
        println!("cargo:rerun-if-changed={}", fbs_file);
    }
    println!("cargo:rerun-if-changed={}", style_dir.display());

    flatc_rust::run(flatc_rust::Args {
        lang: "rust",
        inputs: &fbs_files,
        out_dir: Path::new(&out_dir),
        includes: &[Path::new("src")],
        ..Default::default()
    })
    .expect("flatc schema compilation failed");

    let static_styles = read_toml_file::<StaticConfig>(&style_dir.join("static.toml"))
        .map(|c| c.static_styles)
        .unwrap_or_default();
    let dynamic = read_toml_file::<DynamicConfig>(&style_dir.join("dynamic.toml"))
        .map(|c| c.dynamic)
        .unwrap_or_default();
    let generators = read_toml_file::<GeneratorsConfig>(&style_dir.join("generators.toml"))
        .map(|c| c.generators)
        .unwrap_or_default();
    let screens = read_toml_file::<ScreensConfig>(&style_dir.join("screens.toml"))
        .map(|c| c.screens)
        .unwrap_or_default();
    let states = read_toml_file::<StatesConfig>(&style_dir.join("states.toml"))
        .map(|c| c.states)
        .unwrap_or_default();
    let container_queries =
        read_toml_file::<ContainerQueriesConfig>(&style_dir.join("container_queries.toml"))
            .map(|c| c.container_queries)
            .unwrap_or_default();
    let colors = read_toml_file::<ColorsConfig>(&style_dir.join("colors.toml"))
        .map(|c| c.colors)
        .unwrap_or_default();
    let animation_generators =
        read_toml_file::<AnimationGeneratorsConfig>(&style_dir.join("animation_generators.toml"))
            .map(|c| c.animation_generators)
            .unwrap_or_default();
    let properties_meta = read_toml_file::<PropertiesConfig>(&style_dir.join("property.toml"))
        .map(|c| c.properties)
        .unwrap_or_default();

    // CSS properties from generated data (syntax only by default)
    let props_toml_path = style_dir.join("css").join("properties.toml");
    let css_props_syntax = css_properties_from_toml(&props_toml_path);
    println!(
        "cargo:warning=Loaded {} CSS properties from {}",
        css_props_syntax.len(),
        props_toml_path.display()
    );
    // Merge property names: prefer css-derived syntax
    let mut all_prop_names: Vec<String> = css_props_syntax.keys().cloned().collect();
    for k in properties_meta.keys() {
        if !css_props_syntax.contains_key(k) {
            all_prop_names.push(k.clone());
        }
    }
    all_prop_names.sort();
    all_prop_names.dedup();

    // Compute property abbreviations across the full set
    let prop_abbrs = compute_property_abbreviations(&all_prop_names);

    // Load at-rules, functions, selectors and compute simple minimal unique prefixes
    fn load_names_from_toml(path: &Path) -> Vec<String> {
        let mut names = std::collections::BTreeSet::new();
        if let Some(content) = read_file_string(path) {
            for raw_line in content.lines() {
                let line = raw_line.trim();
                if line.starts_with('[') && line.ends_with(']') {
                    let inner = &line[1..line.len() - 1].trim();
                    // Capture only base section name before any dot
                    if let Some(first_quote) = inner.find('"') {
                        if let Some(second_quote) = inner[first_quote + 1..].find('"') {
                            let name = &inner[first_quote + 1..first_quote + 1 + second_quote];
                            names.insert(name.to_string());
                        }
                    }
                }
            }
        }
        names.into_iter().collect()
    }
    let at_rules_path = style_dir.join("css").join("at-rules.toml");
    let functions_path = style_dir.join("css").join("functions.toml");
    let selectors_path = style_dir.join("css").join("selectors.toml");
    let at_rule_names = load_names_from_toml(&at_rules_path);
    let function_names = load_names_from_toml(&functions_path);
    let selector_names = load_names_from_toml(&selectors_path);
    println!(
        "cargo:warning=Loaded counts — at-rules: {}, functions: {}, selectors: {}",
        at_rule_names.len(),
        function_names.len(),
        selector_names.len()
    );

    fn compute_min_prefix(names: &[String]) -> HashMap<String, String> {
        let mut out = HashMap::new();
        if names.is_empty() {
            return out;
        }
        let cleaned: Vec<String> = names.iter().map(|n| letters_only(n)).collect();
        let mut len = 1usize;
        loop {
            use std::collections::BTreeMap;
            let mut map: BTreeMap<String, Vec<usize>> = BTreeMap::new();
            for (i, c) in cleaned.iter().enumerate() {
                let p = c.chars().take(len).collect::<String>();
                map.entry(p).or_default().push(i);
            }
            let all_unique = map.values().all(|v| v.len() == 1);
            if all_unique {
                for (pref, idxs) in map {
                    out.insert(names[idxs[0]].clone(), pref);
                }
                break;
            }
            len += 1;
            if len > 10 {
                for (i, n) in names.iter().enumerate() {
                    out.insert(n.clone(), cleaned[i].clone());
                }
                break;
            }
        }
        out
    }
    let at_abbr = compute_min_prefix(&at_rule_names);
    let fn_abbr = compute_min_prefix(&function_names);
    let sel_abbr = compute_min_prefix(&selector_names);

    let mut builder = FlatBufferBuilder::new();

    let mut style_offsets = Vec::new();
    for (name, css) in static_styles {
        let name_offset = builder.create_string(&name);
        let css_offset = builder.create_string(&css);
        let table_wip = builder.start_table();
        builder.push_slot(4, name_offset, WIPOffset::new(0));
        builder.push_slot(6, css_offset, WIPOffset::new(0));
        let style_offset = builder.end_table(table_wip);
        style_offsets.push(style_offset);
    }

    let mut dynamic_offsets = Vec::new();
    for (key, values) in dynamic {
        let parts: Vec<&str> = key.split('|').collect();
        if parts.len() != 2 {
            println!(
                "cargo:warning=Invalid dynamic key format in dynamic.toml: '{}'. Skipping.",
                key
            );
            continue;
        }
        let key_name = parts[0];
        let property = parts[1];

        let key_offset = builder.create_string(key_name);
        let property_offset = builder.create_string(property);

        let mut value_offsets = Vec::new();
        for (suffix, value) in values {
            let suffix_offset = builder.create_string(&suffix);
            let value_offset = builder.create_string(&value);
            let table_wip = builder.start_table();
            builder.push_slot(4, suffix_offset, WIPOffset::new(0));
            builder.push_slot(6, value_offset, WIPOffset::new(0));
            let value_offset = builder.end_table(table_wip);
            value_offsets.push(value_offset);
        }
        let values_vec = builder.create_vector(&value_offsets);

        let table_wip = builder.start_table();
        builder.push_slot(4, key_offset, WIPOffset::new(0));
        builder.push_slot(6, property_offset, WIPOffset::new(0));
        builder.push_slot(8, values_vec, WIPOffset::new(0));
        let dynamic_offset = builder.end_table(table_wip);
        dynamic_offsets.push(dynamic_offset);
    }

    let mut generator_offsets = Vec::new();
    for (key, config) in generators {
        let parts: Vec<&str> = key.split('|').collect();
        if parts.len() != 2 {
            println!(
                "cargo:warning=Invalid generator key format in generators.toml: '{}'. Skipping.",
                key
            );
            continue;
        }
        let prefix = parts[0];
        let property = parts[1];

        let prefix_offset = builder.create_string(prefix);
        let property_offset = builder.create_string(property);
        let unit_offset = builder.create_string(&config.unit);

        let table_wip = builder.start_table();
        builder.push_slot(4, prefix_offset, WIPOffset::new(0));
        builder.push_slot(6, property_offset, WIPOffset::new(0));
        builder.push_slot(8, config.multiplier, 0.0f32);
        builder.push_slot(10, unit_offset, WIPOffset::new(0));
        let gen_offset = builder.end_table(table_wip);
        generator_offsets.push(gen_offset);
    }

    let mut screen_offsets = Vec::new();
    for (name, value) in screens {
        let name_offset = builder.create_string(&name);
        let value_offset = builder.create_string(&value);
        let table_wip = builder.start_table();
        builder.push_slot(4, name_offset, WIPOffset::new(0));
        builder.push_slot(6, value_offset, WIPOffset::new(0));
        let screen_offset = builder.end_table(table_wip);
        screen_offsets.push(screen_offset);
    }

    let mut state_offsets = Vec::new();
    for (name, value) in states {
        let name_offset = builder.create_string(&name);
        let value_offset = builder.create_string(&value);
        let table_wip = builder.start_table();
        builder.push_slot(4, name_offset, WIPOffset::new(0));
        builder.push_slot(6, value_offset, WIPOffset::new(0));
        let state_offset = builder.end_table(table_wip);
        state_offsets.push(state_offset);
    }

    let mut cq_offsets = Vec::new();
    for (name, value) in container_queries {
        let name_offset = builder.create_string(&name);
        let value_offset = builder.create_string(&value);
        let table_wip = builder.start_table();
        builder.push_slot(4, name_offset, WIPOffset::new(0));
        builder.push_slot(6, value_offset, WIPOffset::new(0));
        let cq_offset = builder.end_table(table_wip);
        cq_offsets.push(cq_offset);
    }

    let mut color_offsets = Vec::new();
    for (name, value) in colors {
        let name_offset = builder.create_string(&name);
        let value_offset = builder.create_string(&value);
        let table_wip = builder.start_table();
        builder.push_slot(4, name_offset, WIPOffset::new(0));
        builder.push_slot(6, value_offset, WIPOffset::new(0));
        let color_offset = builder.end_table(table_wip);
        color_offsets.push(color_offset);
    }

    let mut anim_gen_offsets = Vec::new();
    for (name, template) in animation_generators {
        let name_offset = builder.create_string(&name);
        let tpl_offset = builder.create_string(&template);
        let table_wip = builder.start_table();
        builder.push_slot(4, name_offset, WIPOffset::new(0));
        builder.push_slot(6, tpl_offset, WIPOffset::new(0));
        let ag_offset = builder.end_table(table_wip);
        anim_gen_offsets.push(ag_offset);
    }

    let styles_vec = builder.create_vector(&style_offsets);
    let dynamic_vec = builder.create_vector(&dynamic_offsets);
    let generators_vec = builder.create_vector(&generator_offsets);
    let screens_vec = builder.create_vector(&screen_offsets);
    let states_vec = builder.create_vector(&state_offsets);
    let cq_vec = builder.create_vector(&cq_offsets);
    let colors_vec = builder.create_vector(&color_offsets);
    let anim_gen_vec = builder.create_vector(&anim_gen_offsets);
    let mut property_offsets = Vec::new();
    for name in all_prop_names {
        // Resolve meta (inherits/initial) and syntax
        let (syntax, inherits, initial) = if let Some(s) = css_props_syntax.get(&name) {
            let meta = properties_meta.get(&name);
            (
                s.as_str(),
                meta.and_then(|m| m.inherits).unwrap_or(false),
                meta.and_then(|m| m.initial_value.clone())
                    .unwrap_or_default(),
            )
        } else if let Some(meta) = properties_meta.get(&name) {
            (
                meta.syntax.as_str(),
                meta.inherits.unwrap_or(false),
                meta.initial_value.clone().unwrap_or_default(),
            )
        } else {
            continue; // shouldn't happen
        };

        // Value keywords and abbreviations
        let keywords = extract_keywords_from_syntax(syntax);
        let value_abbrs = compute_value_abbreviations(&keywords);
        let mut value_offsets = Vec::new();
        for key in keywords {
            if let Some(abbr) = value_abbrs.get(&key) {
                let val_name_offset = builder.create_string(&key);
                let val_abbr_offset = builder.create_string(abbr);
                let v_wip = builder.start_table();
                // PropertyValue { name, abbr }
                builder.push_slot(4, val_name_offset, WIPOffset::new(0));
                builder.push_slot(6, val_abbr_offset, WIPOffset::new(0));
                let v_off = builder.end_table(v_wip);
                value_offsets.push(v_off);
            }
        }
        let values_vec = builder.create_vector(&value_offsets);

        let name_offset = builder.create_string(&name);
        let syntax_offset = builder.create_string(syntax);
        let initial_offset = builder.create_string(&initial);
        let abbr_str = prop_abbrs.get(&name).cloned().unwrap_or_else(|| {
            letters_only(&name)
                .chars()
                .next()
                .unwrap_or('x')
                .to_string()
        });
        let abbr_offset = builder.create_string(&abbr_str);
        let table_wip = builder.start_table();
        builder.push_slot(4, name_offset, WIPOffset::new(0));
        builder.push_slot(6, syntax_offset, WIPOffset::new(0));
        builder.push_slot(8, inherits, false);
        builder.push_slot(10, initial_offset, WIPOffset::new(0));
        builder.push_slot(12, abbr_offset, WIPOffset::new(0));
        builder.push_slot(14, values_vec, WIPOffset::new(0));
        let prop_offset = builder.end_table(table_wip);
        property_offsets.push(prop_offset);
    }
    let properties_vec = builder.create_vector(&property_offsets);

    // At-rule abbrev entries
    let mut at_offsets = Vec::new();
    for name in &at_rule_names {
        if let Some(a) = at_abbr.get(name) {
            let n_off = builder.create_string(name);
            let a_off = builder.create_string(a);
            let wip = builder.start_table();
            builder.push_slot(4, n_off, WIPOffset::new(0));
            builder.push_slot(6, a_off, WIPOffset::new(0));
            at_offsets.push(builder.end_table(wip));
        }
    }
    let at_vec = builder.create_vector(&at_offsets);

    // Function abbrev entries
    let mut fn_offsets = Vec::new();
    for name in &function_names {
        if let Some(a) = fn_abbr.get(name) {
            let n_off = builder.create_string(name);
            let a_off = builder.create_string(a);
            let wip = builder.start_table();
            builder.push_slot(4, n_off, WIPOffset::new(0));
            builder.push_slot(6, a_off, WIPOffset::new(0));
            fn_offsets.push(builder.end_table(wip));
        }
    }
    let fn_vec = builder.create_vector(&fn_offsets);

    // Selector abbrev entries
    let mut sel_offsets = Vec::new();
    for name in &selector_names {
        if let Some(a) = sel_abbr.get(name) {
            let n_off = builder.create_string(name);
            let a_off = builder.create_string(a);
            let wip = builder.start_table();
            builder.push_slot(4, n_off, WIPOffset::new(0));
            builder.push_slot(6, a_off, WIPOffset::new(0));
            sel_offsets.push(builder.end_table(wip));
        }
    }
    let sel_vec = builder.create_vector(&sel_offsets);
    let base_css = fs::read_to_string(style_dir.join("base.css")).unwrap_or_default();
    let base_css_offset = builder.create_string(&base_css);
    let property_css = fs::read_to_string(style_dir.join("property.css")).unwrap_or_default();
    let property_css_offset = builder.create_string(&property_css);

    let table_wip = builder.start_table();
    builder.push_slot(4, styles_vec, WIPOffset::new(0));
    builder.push_slot(6, generators_vec, WIPOffset::new(0));
    builder.push_slot(8, dynamic_vec, WIPOffset::new(0));
    builder.push_slot(10, screens_vec, WIPOffset::new(0));
    builder.push_slot(12, states_vec, WIPOffset::new(0));
    builder.push_slot(14, cq_vec, WIPOffset::new(0));
    builder.push_slot(16, colors_vec, WIPOffset::new(0));
    builder.push_slot(18, anim_gen_vec, WIPOffset::new(0));
    builder.push_slot(20, properties_vec, WIPOffset::new(0));
    builder.push_slot(22, base_css_offset, WIPOffset::new(0));
    builder.push_slot(24, property_css_offset, WIPOffset::new(0));
    builder.push_slot(26, at_vec, WIPOffset::new(0));
    builder.push_slot(28, fn_vec, WIPOffset::new(0));
    builder.push_slot(30, sel_vec, WIPOffset::new(0));
    let config_root = builder.end_table(table_wip);

    builder.finish(config_root, None);

    let buf = builder.finished_data();
    let styles_bin_path = style_dir.join("style.bin");
    fs::create_dir_all(styles_bin_path.parent().unwrap()).expect("Failed to create .dx directory");
    let needs_write = match fs::read(&styles_bin_path) {
        Ok(existing) => existing.as_slice() != buf,
        Err(_) => true,
    };
    if needs_write {
        match fs::write(&styles_bin_path, buf) {
            Ok(_) => {}
            Err(e) => {
                if let Some(code) = e.raw_os_error() {
                    if code == 1224 {
                        let tmp = styles_bin_path.with_extension("bin.new");
                        if fs::write(&tmp, buf).is_ok() {
                            let _ = fs::rename(&tmp, &styles_bin_path);
                        } else {
                        }
                    } else {
                        panic!("Failed to write style.bin: {:?}", e);
                    }
                } else {
                    panic!("Failed to write style.bin: {:?}", e);
                }
            }
        }
    }

    println!(
        "cargo:rustc-env=DX_STYLE_BIN={}",
        style_dir.join("style.bin").to_string_lossy()
    );
}
