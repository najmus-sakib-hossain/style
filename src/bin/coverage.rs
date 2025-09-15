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
    utilities: BTreeMap<String, Vec<String>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let spec: Spec = serde_json::from_str(&fs::read_to_string("tailwind-spec.json")?)?;
    let mut tw = BTreeSet::new();
    for list in spec.utilities.values() {
        for k in list {
            tw.insert(k.clone());
        }
    }

    let missing: Vec<_> = tw.difference(&all).cloned().collect();
    let extra: Vec<_> = all.difference(&tw).cloned().collect();

    println!("Total utilities configured: {}", all.len());
    println!(
        "Tailwind spec ({}): total utility keys referenced: {}",
        spec.version,
        tw.len()
    );
    if !missing.is_empty() {
        println!("Missing: {}", missing.join(", "));
    } else {
        println!("No missing utilities.");
    }
    println!("Extra (not in spec list):");
    for e in extra {
        println!("  {e}");
    }
    Ok(())
}
