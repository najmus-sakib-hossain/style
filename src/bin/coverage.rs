use serde::Deserialize;
use std::{collections::BTreeSet, fs, path::Path};

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

    // Placeholder Tailwind v4.1 list stub (user should replace with authoritative JSON)
    let tailwind_stub: &[&str] = &[
        "p",
        "m",
        "mx",
        "my",
        "ms",
        "me",
        "ps",
        "pe",
        "size",
        "w",
        "h",
        "leading",
        "tracking",
        "font-stretch",
        "font-optical",
        "mask-type",
        "mask-mode",
        "mask-repeat",
        "mask-position",
        "mask-size",
        "writing-mode",
        "text-orientation",
        "text-wrap",
        "overflow",
        "overflow-x",
        "overflow-y",
        "scrollbar",
    ];
    let tw: BTreeSet<_> = tailwind_stub.iter().cloned().collect();

    let missing: Vec<_> = tw.difference(&all).cloned().collect();
    let extra: Vec<_> = all.difference(&tw).cloned().collect();

    println!("Total utilities configured: {}", all.len());
    println!("Stub Tailwind list size: {}", tw.len());
    if !missing.is_empty() {
        println!("Missing (stub subset): {}", missing.join(", "));
    } else {
        println!("No missing utilities from stub subset.");
    }
    println!("Extra (not in stub subset): {}");
    for e in extra {
        println!("  {e}");
    }
    Ok(())
}
