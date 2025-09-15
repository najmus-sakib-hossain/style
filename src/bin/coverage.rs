use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

// Simple structures to load our toml segments
#[derive(Deserialize, Debug)]
struct Generators {
    #[serde(flatten)]
    map: toml::value::Table,
}
#[derive(Deserialize, Debug)]
struct Dynamic {
    #[serde(flatten)]
    map: toml::value::Table,
}
#[derive(Deserialize, Debug)]
struct Static {
    #[serde(flatten)]
    map: toml::value::Table,
}

#[derive(Deserialize)]
struct Spec {
    version: String,
    utilities: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    variants: Vec<String>,
}

// Utility: gather suffix map from dynamic or generators entries; returns mapping util -> suffix set
fn collect_suffixes(
    dyns: &toml::Value,
    generators: &toml::Value,
    targets: &[&str],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    if let Some(dtab) = dyns.get("dynamic").and_then(|v| v.as_table()) {
        for (full_key, val_map) in dtab {
            if let Some((util, _prop)) = full_key.split_once('|') {
                if targets.contains(&util) {
                    if let Some(map) = val_map.as_table() {
                        let set = out.entry(util.to_string()).or_default();
                        for suffix in map.keys() {
                            set.insert(suffix.clone());
                        }
                    }
                }
            }
        }
    }
    if let Some(gtab) = generators.get("generators").and_then(|v| v.as_table()) {
        for (full_key, _cfg) in gtab {
            if let Some((util, _prop)) = full_key.split_once('|') {
                if targets.contains(&util) {
                    // Generators are numeric; we can't enumerate all numeric possibilities. We'll insert a marker.
                    out.entry(util.to_string())
                        .or_default()
                        .insert("<num>".into());
                }
            }
        }
    }
    out
}

fn detect_arbitrary_classes(all: &BTreeSet<String>) -> Vec<String> {
    // Very naive: if class contains '[' and ']' treat as arbitrary.
    all.iter()
        .filter(|c| c.contains('[') && c.contains(']'))
        .cloned()
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple CLI flags (manual parse):
    // --hide-missing-values : suppress detailed missing value lines
    // --generator-covers <csv list of value set keys> : treat listed value sets as fully covered if a numeric generator exists
    // --hide-missing-utilities : suppress missing utility list
    // --json : output machine-readable JSON (suppresses normal text except errors)
    // --only-category <name> : restrict category coverage output to a single category
    // --fail-on-missing [utilities|values|variants|any] : exit 1 if gaps detected (default any)
    // Container query synthesis: we replicate build.rs cq-* variant injection (simple list here)
    let args: Vec<String> = std::env::args().collect();
    let hide_missing_values = args.iter().any(|a| a == "--hide-missing-values");
    let hide_missing_utilities = args.iter().any(|a| a == "--hide-missing-utilities");
    let json_mode = args.iter().any(|a| a == "--json");
    let only_category = args
        .iter()
        .position(|a| a == "--only-category")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.to_string());
    let mut generator_cover_sets: BTreeSet<String> = BTreeSet::new();
    let explicit_generator_flag = args.iter().any(|a| a == "--generator-covers");
    if let Some(idx) = args.iter().position(|a| a == "--generator-covers") {
        if let Some(list) = args.get(idx + 1) {
            for part in list.split(',') {
                if !part.trim().is_empty() {
                    generator_cover_sets.insert(part.trim().to_string());
                }
            }
        }
    }
    // --fail-on-missing (optional value: utilities|values|any). Default any
    let mut fail_mode = None; // Some("utilities"|"values"|"any")
    if let Some(idx) = args.iter().position(|a| a == "--fail-on-missing") {
        // Look ahead for value not starting with '--'
        if let Some(val) = args.get(idx + 1) {
            if !val.starts_with('-') {
                fail_mode = Some(val.as_str());
            } else {
                fail_mode = Some("any");
            }
        } else {
            fail_mode = Some("any");
        }
    }
    let root = Path::new(".dx/style");
    let gens: toml::Value = toml::from_str(&fs::read_to_string(root.join("generators.toml"))?)?;
    let dyns: toml::Value = toml::from_str(&fs::read_to_string(root.join("dynamic.toml"))?)?;
    let stats: toml::Value = toml::from_str(&fs::read_to_string(root.join("static.toml"))?)?;

    let mut all = BTreeSet::new();
    if let Some(g) = gens.get("generators").and_then(|v| v.as_table()) {
        for k in g.keys() {
            all.insert(k.clone());
        }
    }
    if let Some(d) = dyns.get("dynamic").and_then(|v| v.as_table()) {
        for k in d.keys() {
            all.insert(k.clone());
        }
    }
    if let Some(s) = stats.get("static").and_then(|v| v.as_table()) {
        for k in s.keys() {
            all.insert(k.clone());
        }
    }

    let mut spec: Spec = serde_json::from_str(&fs::read_to_string("tailwind-spec.json")?)?;
    // Merge any supplemental spec JSON files under spec-extra/*.json
    if let Ok(entries) = fs::read_dir("spec-extra") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(txt) = fs::read_to_string(&path) {
                    if let Ok(extra_spec) = serde_json::from_str::<serde_json::Value>(&txt) {
                        if let Some(obj) = extra_spec.as_object() {
                            if let Some(utilities) =
                                obj.get("utilities").and_then(|v| v.as_object())
                            {
                                for (cat, val) in utilities {
                                    let entry = spec
                                        .utilities
                                        .entry(cat.clone())
                                        .or_insert_with(|| serde_json::json!([]));
                                    if let (Some(dst_arr), Some(src_arr)) =
                                        (entry.as_array_mut(), val.as_array())
                                    {
                                        let existing: BTreeSet<String> = dst_arr
                                            .iter()
                                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                            .collect();
                                        for item in src_arr {
                                            if let Some(s) = item.as_str() {
                                                if !existing.contains(s) {
                                                    dst_arr.push(serde_json::Value::String(
                                                        s.to_string(),
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if let Some(extra_variants) =
                                obj.get("variants").and_then(|v| v.as_array())
                            {
                                let mut existing: BTreeSet<String> =
                                    spec.variants.iter().cloned().collect();
                                for v in extra_variants {
                                    if let Some(s) = v.as_str() {
                                        if existing.insert(s.to_string()) {
                                            spec.variants.push(s.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Synthesize container query variants (cq-*) so they are treated as implemented variants
    let synthesized_cq: [&str; 13] = [
        "cq-xs", "cq-sm", "cq-md", "cq-lg", "cq-xl", "cq-2xl", "cq-3xl", "cq-4xl", "cq-5xl",
        "cq-6xl", "cq-7xl", "cq-8xl", "cq-9xl",
    ];
    for k in synthesized_cq {
        // Historically these were inserted as utilities. We keep that for backward compatibility
        all.insert(k.to_string());
    }
    let mut tw = BTreeSet::new();
    // Collect utility names from categories (skip special 'values' map)
    for (cat, val) in &spec.utilities {
        if cat == "values" {
            continue;
        }
        if let Some(arr) = val.as_array() {
            for k in arr {
                if let Some(s) = k.as_str() {
                    tw.insert(s.to_string());
                }
            }
        }
    }

    let missing: Vec<_> = tw.difference(&all).cloned().collect();
    let extra: Vec<_> = all.difference(&tw).cloned().collect();

    // Build per-category detail structure
    #[derive(serde::Serialize)]
    struct CategoryReport {
        present: Vec<String>,
        missing: Vec<String>,
        total: usize,
        covered: usize,
        percent: f64,
    }
    #[derive(serde::Serialize)]
    struct ValueDiffEntry {
        ok: bool,
        covered: usize,
        spec_total: usize,
        via_generator: bool,
        missing: Vec<String>,
        extra: Vec<String>,
    }
    #[derive(serde::Serialize)]
    struct VariantReport {
        present: Vec<String>,
        missing: Vec<String>,
        extra: Vec<String>,
        total_spec: usize,
        // matrix summary: variant -> number of utilities implemented (approximate)
        matrix_counts: BTreeMap<String, usize>,
    }
    #[derive(serde::Serialize)]
    struct OutputJson {
        version: String,
        totals: serde_json::Value,
        categories: BTreeMap<String, CategoryReport>,
        values: BTreeMap<String, ValueDiffEntry>,
        variants: VariantReport,
        arbitrary_count: usize,
    }

    let mut category_reports: BTreeMap<String, CategoryReport> = BTreeMap::new();

    if !json_mode {
        println!("Total utilities configured: {}", all.len());
        println!(
            "Tailwind spec ({}): total utility keys referenced: {}",
            spec.version,
            tw.len()
        );
    }

    // Per-category coverage
    if !json_mode {
        println!("\nPer-category coverage:");
    }
    for (cat, val) in &spec.utilities {
        if cat == "values" {
            continue;
        }
        if let Some(arr) = val.as_array() {
            if let Some(ref only) = only_category {
                if cat != only {
                    continue;
                }
            }
            let total = arr.len();
            let mut present_list = Vec::new();
            let mut missing_list = Vec::new();
            for k in arr {
                if let Some(s) = k.as_str() {
                    if all.contains(s) {
                        present_list.push(s.to_string());
                    } else {
                        missing_list.push(s.to_string());
                    }
                }
            }
            let covered = present_list.len();
            let percent = if total > 0 {
                covered as f64 / total as f64 * 100.0
            } else {
                100.0
            };
            if !json_mode {
                println!("  {cat}: {covered}/{total} ({percent:.1}%)");
            }
            category_reports.insert(
                cat.clone(),
                CategoryReport {
                    present: present_list,
                    missing: missing_list,
                    total,
                    covered,
                    percent,
                },
            );
        }
    }
    if !json_mode {
        if !hide_missing_utilities {
            if !missing.is_empty() {
                println!("\nMissing: {}", missing.join(", "));
            } else {
                println!("\nNo missing utilities.");
            }
        }
        if !extra.is_empty() {
            println!("Extra (not in spec list):");
            for e in &extra {
                println!("  {e}");
            }
        }
    }

    // Basic fraction/percentage validation against dynamic keys captured from spec tokens
    // We approximate by inferring value sets from class fragments in 'all' (suffix patterns not stored separately here).
    // Placeholder: real implementation would need to inspect dynamic.toml for allowed suffixes.
    // (Future enhancement) parse dynamic.toml again and build per-key suffix sets.

    // NOTE: Current architecture doesn't expose suffixes separately, so we skip deep validation.
    // Per-value spec (fractions) summary if present
    if !json_mode {
        if let Some(values_val) = spec.utilities.get("values") {
            if let Some(obj) = values_val.as_object() {
                println!("\nValue sets (spec counts):");
                for (util, arr) in obj {
                    if let Some(a) = arr.as_array() {
                        println!("  {util}: {} values in spec", a.len());
                    }
                }
                println!(
                    "(Detailed per-value gap detection requires parsing dynamic.toml suffixes)"
                );
            }
        }
    }

    // If user did not explicitly provide generator covers, attempt auto-detection by scanning generators keys
    if !explicit_generator_flag {
        if let Some(gtab) = gens.get("generators").and_then(|v| v.as_table()) {
            for (full_key, _cfg) in gtab {
                if let Some((util, _prop)) = full_key.split_once('|') {
                    // heuristics: sizing, spacing, radii, line-height, logical sizes
                    match util {
                        "w" | "h" | "size" | "m" | "p" | "mx" | "my" | "mt" | "mr" | "mb"
                        | "ml" | "pt" | "pr" | "pb" | "pl" | "gap" | "gap-x" | "gap-y"
                        | "space-x" | "space-y" | "leading" | "rounded" | "rounded-t"
                        | "rounded-r" | "rounded-b" | "rounded-l" | "rounded-s" | "rounded-e"
                        | "rounded-ss" | "rounded-se" | "rounded-es" | "rounded-ee" | "is"
                        | "bs" | "min-is" | "min-bs" | "max-is" | "max-bs" => {
                            generator_cover_sets.insert(util.to_string());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Deep value diff for selected utilities (extended set)
    let mut value_reports: BTreeMap<String, ValueDiffEntry> = BTreeMap::new();
    if let Some(values_val) = spec.utilities.get("values") {
        if let Some(obj) = values_val.as_object() {
            if !json_mode {
                println!("\nPer-value diffs:");
            }
            let target_utils: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
            let configured = collect_suffixes(&dyns, &gens, &target_utils);
            for util in target_utils {
                if let Some(spec_arr) = obj.get(util).and_then(|v| v.as_array()) {
                    let spec_set: BTreeSet<String> = spec_arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    let mut cfg_set = configured.get(util).cloned().unwrap_or_default();
                    let had_numeric = cfg_set.remove("<num>");
                    let mut missing: Vec<_> = spec_set.difference(&cfg_set).cloned().collect();
                    let extra: Vec<_> = cfg_set.difference(&spec_set).cloned().collect();
                    if had_numeric && generator_cover_sets.contains(util) {
                        missing.clear();
                    }
                    let ok = missing.is_empty() && extra.is_empty();
                    if !json_mode {
                        if ok {
                            if had_numeric {
                                if generator_cover_sets.contains(util) {
                                    println!("  {util}: OK ({} via generator)", spec_set.len());
                                } else {
                                    println!("  {util}: OK ({}+ numeric)", spec_set.len());
                                }
                            } else {
                                println!("  {util}: OK ({} values)", spec_set.len());
                            }
                        } else {
                            if !missing.is_empty() && !hide_missing_values {
                                println!("  {util}: missing -> {}", missing.join(" "));
                            }
                            if !extra.is_empty() {
                                println!("  {util}: extra -> {}", extra.join(" "));
                            }
                            if had_numeric {
                                println!("  {util}: note -> numeric generator present");
                            }
                        }
                    }
                    value_reports.insert(
                        util.to_string(),
                        ValueDiffEntry {
                            ok,
                            covered: spec_set.len() - missing.len(),
                            spec_total: spec_set.len(),
                            via_generator: had_numeric && generator_cover_sets.contains(util),
                            missing,
                            extra,
                        },
                    );
                }
            }
        }
    }

    // Arbitrary value detection
    let arbitrary = detect_arbitrary_classes(&all);

    // Variant coverage: diff spec.variants (if provided) against states.toml keys plus synthesized cq-* list.
    let mut implemented_variants: BTreeSet<String> = BTreeSet::new();
    // Load states.toml directly (already read earlier? we only read gens/dyns/stats; read states here)
    if let Ok(states_txt) = fs::read_to_string(root.join("states.toml")) {
        if let Ok(state_val) = toml::from_str::<toml::Value>(&states_txt) {
            if let Some(stab) = state_val.get("states").and_then(|v| v.as_table()) {
                for k in stab.keys() {
                    implemented_variants.insert(k.to_string());
                }
            }
        }
    }
    for cq in synthesized_cq {
        implemented_variants.insert(cq.to_string());
    }
    let spec_variants: BTreeSet<String> = spec.variants.iter().cloned().collect();
    let variant_missing: Vec<_> = spec_variants
        .difference(&implemented_variants)
        .cloned()
        .collect();
    let variant_present: Vec<_> = spec_variants
        .intersection(&implemented_variants)
        .cloned()
        .collect();
    let variant_extra: Vec<_> = implemented_variants
        .difference(&spec_variants)
        .cloned()
        .collect();
    // Matrix counts heuristic: count utilities that appear in spec for each variant prefix form 'variant:utility'
    let mut matrix_counts: BTreeMap<String, usize> = BTreeMap::new();
    if !spec_variants.is_empty() {
        for v in &implemented_variants {
            let mut count = 0usize;
            for util in &tw {
                // spec utilities list
                let composite = format!("{v}:{util}");
                if all.contains(&composite) {
                    count += 1;
                }
            }
            matrix_counts.insert(v.clone(), count);
        }
    }
    if !json_mode {
        if !spec_variants.is_empty() {
            println!(
                "\nVariants: {}/{} ({} missing)",
                variant_present.len(),
                spec_variants.len(),
                variant_missing.len()
            );
        } else {
            println!("\nVariants: (no variant expectations in spec)");
        }
    }
    if !json_mode {
        if !arbitrary.is_empty() {
            println!(
                "\nArbitrary classes detected ({}). Skipped from value diff.",
                arbitrary.len()
            );
        }
    }

    let mut fail = false;
    if let Some(mode) = fail_mode {
        match mode {
            "utilities" => {
                if !missing.is_empty() {
                    fail = true;
                }
            }
            "values" => {
                if value_reports.values().any(|r| !r.ok) {
                    fail = true;
                }
            }
            "variants" => {
                if !variant_missing.is_empty() {
                    fail = true;
                }
            }
            _ => {
                // any
                if !missing.is_empty()
                    || value_reports.values().any(|r| !r.ok)
                    || !variant_missing.is_empty()
                {
                    fail = true;
                }
            }
        }
    }

    if json_mode {
        let out = OutputJson {
            version: spec.version,
            totals: serde_json::json!({
                "configured": all.len(),
                "spec_utilities": tw.len(),
                "missing_utilities": missing.len(),
                "extra_utilities": extra.len()
            }),
            categories: category_reports,
            values: value_reports,
            variants: VariantReport {
                present: variant_present,
                missing: variant_missing.clone(),
                extra: variant_extra,
                total_spec: spec_variants.len(),
                matrix_counts,
            },
            arbitrary_count: arbitrary.len(),
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    }
    if fail {
        std::process::exit(1);
    }
    Ok(())
}
