#![allow(dead_code)] // Color module API surface is broader than current in-crate usage.

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
