#[allow(dead_code)]
mod style_generated {
    #![allow(
        dead_code,
        unused_imports,
        non_snake_case,
        clippy::all,
        unsafe_op_in_unsafe_fn,
        mismatched_lifetime_syntaxes
    )]
    include!(concat!(env!("OUT_DIR"), "/style_generated.rs"));
}

use anyhow::{Context, Result};
use memmap2::Mmap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use style_generated::style_schema;

fn main() -> Result<()> {
    // Locate style.bin (use DX_STYLE_BIN env if set; else default relative to repo)
    let override_path = std::env::var("DX_STYLE_BIN").ok();
    let try_paths = [
        override_path.as_deref().unwrap_or(""),
        ".dx/style/style.bin",
        "../../.dx/style/style.bin",
    ];
    let mut opened: Option<File> = None;
    let mut tried: Vec<String> = Vec::new();
    for p in try_paths.iter().filter(|p| !p.is_empty()) {
        let path = Path::new(p);
        tried.push(path.display().to_string());
        if let Ok(f) = File::open(path) {
            opened = Some(f);
            break;
        }
    }
    let file = opened.with_context(|| format!("opening style.bin; tried: {}", tried.join(", ")))?;
    let mmap = unsafe { Mmap::map(&file) }.context("mmapping style.bin")?;
    let config = flatbuffers::root::<style_schema::Config>(&mmap)
        .context("parsing style.bin as FlatBuffer")?;

    let mut out = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("output.log")
        .context("opening output.log")?;

    writeln!(out, "[Properties]")?;
    let mut prop_count = 0usize;
    let mut has_display = false;
    if let Some(props) = config.properties() {
        for p in props {
            let name = p.name();
            let abbr = p.abbr().unwrap_or("");
            if name == "display" {
                has_display = true;
            }
            prop_count += 1;
            writeln!(out, "{} => {}", abbr, name)?;
            if let Some(vals) = p.values() {
                for v in vals {
                    writeln!(out, "  {} => {}", v.abbr(), v.name())?;
                }
            }
        }
    }
    writeln!(out, "\n[Properties: total={}]", prop_count)?;
    writeln!(
        out,
        "display-present: {}",
        if has_display { "yes" } else { "no" }
    )?;

    writeln!(out, "\n[At-Rules]")?;
    let mut at_count = 0usize;
    if let Some(list) = config.at_rules() {
        for r in list {
            at_count += 1;
            writeln!(out, "{} => {}", r.abbr(), r.name())?;
        }
    }
    writeln!(out, "[At-Rules: total={}]", at_count)?;

    writeln!(out, "\n[Functions]")?;
    let mut fn_count = 0usize;
    if let Some(list) = config.functions() {
        for f in list {
            fn_count += 1;
            writeln!(out, "{} => {}", f.abbr(), f.name())?;
        }
    }
    writeln!(out, "[Functions: total={}]", fn_count)?;

    writeln!(out, "\n[Selectors]")?;
    let mut sel_count = 0usize;
    if let Some(list) = config.selectors() {
        for s in list {
            sel_count += 1;
            writeln!(out, "{} => {}", s.abbr(), s.name())?;
        }
    }
    writeln!(out, "[Selectors: total={}]", sel_count)?;

    Ok(())
}
