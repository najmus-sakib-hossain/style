#![allow(dead_code)] // Color module API surface is broader than current in-crate usage.

use crate::core::color::color::{Argb, Oklch};
use crate::core::engine::StyleEngine;

#[cfg(all(feature = "image", not(feature = "std")))]
compile_error!("\"image\" feature requires \"std\" feature");

#[cfg(all(feature = "std", feature = "libm"))]
compile_error!("features \"std\" and \"libm\" cannot be enabled simultaneously");

#[cfg(all(not(feature = "std"), not(feature = "libm")))]
compile_error!("\"no-std\" requires \"libm\" feature");

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
pub(crate) use ahash::HashMap as Map;
#[cfg(not(feature = "std"))]
pub(crate) use alloc::collections::BTreeMap as Map;

#[allow(dead_code)]
pub(crate) type IndexMap<K, V> =
    indexmap::IndexMap<K, V, core::hash::BuildHasherDefault<ahash::AHasher>>;

pub mod blend;
pub mod color;
pub mod contrast;
pub mod dislike;
pub mod dynamic_color;
pub mod error;
pub mod hct;
#[cfg(feature = "image")]
pub mod image;
pub mod palette;
pub mod quantize;
pub mod scheme;
pub mod score;
pub mod temperature;
pub mod theme;
pub mod utils;

pub use error::Error;

pub fn generate_color_css(engine: &StyleEngine, class_name: &str) -> Option<String> {
    if let Some(name) = class_name.strip_prefix("bg-") {
        if derive_color_value(engine, name).is_some() {
            return Some(format!("background-color: var(--color-{})", name));
        }
    }
    if let Some(name) = class_name.strip_prefix("text-") {
        if derive_color_value(engine, name).is_some() {
            return Some(format!("color: var(--color-{})", name));
        }
    }
    None
}

// --- Dynamic color token support -----------------------------------------------------------
// Any class "bg-<token>" or "text-<token>" will now produce a CSS variable --color-<token>.
// 1. If <token> exists in colors.toml it uses that value.
// 2. Else if <token> matches a CSS color keyword we emit that keyword.
// 3. Else if <token> is a 3/4/6/8 length hex sequence we emit #<token>.
// Otherwise we skip generation (invalid color).

// Sorted for binary_search
const CSS_COLOR_KEYWORDS: &[&str] = &[
    "aliceblue",
    "antiquewhite",
    "aqua",
    "aquamarine",
    "azure",
    "beige",
    "bisque",
    "black",
    "blanchedalmond",
    "blue",
    "blueviolet",
    "brown",
    "burlywood",
    "cadetblue",
    "chartreuse",
    "chocolate",
    "coral",
    "cornflowerblue",
    "cornsilk",
    "crimson",
    "cyan",
    "darkblue",
    "darkcyan",
    "darkgoldenrod",
    "darkgray",
    "darkgreen",
    "darkgrey",
    "darkkhaki",
    "darkmagenta",
    "darkolivegreen",
    "darkorange",
    "darkorchid",
    "darkred",
    "darksalmon",
    "darkseagreen",
    "darkslateblue",
    "darkslategray",
    "darkslategrey",
    "darkturquoise",
    "darkviolet",
    "deeppink",
    "deepskyblue",
    "dimgray",
    "dimgrey",
    "dodgerblue",
    "firebrick",
    "floralwhite",
    "forestgreen",
    "fuchsia",
    "gainsboro",
    "ghostwhite",
    "gold",
    "goldenrod",
    "gray",
    "green",
    "greenyellow",
    "grey",
    "honeydew",
    "hotpink",
    "indianred",
    "indigo",
    "ivory",
    "khaki",
    "lavender",
    "lavenderblush",
    "lawngreen",
    "lemonchiffon",
    "lightblue",
    "lightcoral",
    "lightcyan",
    "lightgoldenrodyellow",
    "lightgray",
    "lightgreen",
    "lightgrey",
    "lightpink",
    "lightsalmon",
    "lightseagreen",
    "lightskyblue",
    "lightslategray",
    "lightslategrey",
    "lightsteelblue",
    "lightyellow",
    "lime",
    "limegreen",
    "linen",
    "magenta",
    "maroon",
    "mediumaquamarine",
    "mediumblue",
    "mediumorchid",
    "mediumpurple",
    "mediumseagreen",
    "mediumslateblue",
    "mediumspringgreen",
    "mediumturquoise",
    "mediumvioletred",
    "midnightblue",
    "mintcream",
    "mistyrose",
    "moccasin",
    "navajowhite",
    "navy",
    "oldlace",
    "olive",
    "olivedrab",
    "orange",
    "orangered",
    "orchid",
    "palegoldenrod",
    "palegreen",
    "paleturquoise",
    "palevioletred",
    "papayawhip",
    "peachpuff",
    "peru",
    "pink",
    "plum",
    "powderblue",
    "purple",
    "rebeccapurple",
    "red",
    "rosybrown",
    "royalblue",
    "saddlebrown",
    "salmon",
    "sandybrown",
    "seagreen",
    "seashell",
    "sienna",
    "silver",
    "skyblue",
    "slateblue",
    "slategray",
    "slategrey",
    "snow",
    "springgreen",
    "steelblue",
    "tan",
    "teal",
    "thistle",
    "tomato",
    "turquoise",
    "violet",
    "wheat",
    "white",
    "whitesmoke",
    "yellow",
    "yellowgreen",
    "transparent",
    "currentcolor",
];

pub fn derive_color_value(engine: &StyleEngine, name: &str) -> Option<String> {
    if let Some(v) = engine.colors.get(name) {
        return Some(v.clone());
    }
    let lower = name.to_ascii_lowercase();
    if CSS_COLOR_KEYWORDS.binary_search(&lower.as_str()).is_ok() {
        return Some(lower);
    }
    let len = lower.len();
    if matches!(len, 3 | 4 | 6 | 8) && lower.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(format!("#{}", lower));
    }
    None
}

fn parse_oklch_value(value: &str) -> Option<Oklch> {
    let trimmed = value.trim();
    let inner = trimmed
        .strip_prefix("oklch(")?
        .strip_suffix(')')?
        .replace('/', " ");
    let mut parts = inner
        .split_whitespace()
        .filter(|segment| !segment.is_empty());
    let l_raw = parts.next()?;
    let c_raw = parts.next()?;
    let h_raw = parts.next()?;

    fn parse_component(component: &str) -> Option<f64> {
        let cleaned = component.trim_end_matches(|ch: char| {
            ch == '%' || ch == '°' || ch == 'd' || ch == 'e' || ch == 'g'
        });
        if cleaned.is_empty() {
            return None;
        }
        cleaned.parse::<f64>().ok()
    }

    let mut l_value = parse_component(l_raw)?;
    if l_value > 1.0 {
        l_value /= 100.0;
    }
    let c_value = parse_component(c_raw)?;
    let h_value = parse_component(h_raw)?;

    Some(Oklch {
        l: l_value,
        c: c_value,
        h: h_value,
    })
}

pub(crate) fn parse_color_to_argb(value: &str) -> Option<Argb> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower == "transparent" || lower == "currentcolor" || lower == "inherit" {
        return None;
    }
    if trimmed.starts_with("oklch(") {
        return parse_oklch_value(trimmed).map(Argb::from);
    }
    trimmed.parse::<Argb>().ok()
}

pub(crate) fn format_argb_as_oklch(color: Argb) -> String {
    let oklch = Oklch::from(color);
    format!("oklch({:.2} {:.3} {:.2})", oklch.l, oklch.c, oklch.h)
}

pub(crate) fn normalize_color_to_oklch(value: &str) -> Option<String> {
    parse_color_to_argb(value).map(format_argb_as_oklch)
}
