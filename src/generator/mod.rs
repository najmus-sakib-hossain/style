use cssparser::serialize_identifier;

use crate::core::{
    AppState, group::GroupRegistry, properties_layer_present, set_properties_layer_present,
};

#[allow(dead_code)]
pub fn generate_css_into<'a, I>(buf: &mut Vec<u8>, classes: I, groups: &mut GroupRegistry)
where
    I: IntoIterator<Item = &'a String>,
{
    let mut escaped = String::with_capacity(64);
    let engine_opt = std::panic::catch_unwind(|| AppState::engine()).ok();
    if let Some(engine) = engine_opt {
        let collected: Vec<&String> = classes.into_iter().collect();
        if buf.is_empty() && !properties_layer_present() {
            let props = engine.property_at_rules();
            if !props.is_empty() {
                buf.extend_from_slice(props.as_bytes());
                set_properties_layer_present();
            }
            let (root_vars, dark_vars) =
                engine.generate_color_vars_for(collected.iter().map(|s| *s));
            if !root_vars.is_empty() {
                buf.extend_from_slice(root_vars.as_bytes());
            }
            if !dark_vars.is_empty() {
                buf.extend_from_slice(dark_vars.as_bytes());
            }
        }
        for class in collected {
            if groups.is_internal_token(class) {
                continue;
            }
            if let Some(alias_css) = groups.generate_css_for(class, engine) {
                buf.extend_from_slice(alias_css.as_bytes());
                if !alias_css.ends_with('\n') {
                    buf.push(b'\n');
                }
                continue;
            }
            if let Some(css) = engine.css_for_class(class) {
                buf.extend_from_slice(css.as_bytes());
                if !css.ends_with('\n') {
                    buf.push(b'\n');
                }
            } else {
                buf.push(b'.');
                escaped.clear();
                serialize_identifier(class, &mut escaped).unwrap();
                buf.extend_from_slice(escaped.as_bytes());
                buf.extend_from_slice(b" {}\n");
            }
        }
    } else {
        for class in classes {
            if groups.is_internal_token(class) {
                continue;
            }
            buf.push(b'.');
            escaped.clear();
            serialize_identifier(class, &mut escaped).unwrap();
            buf.extend_from_slice(escaped.as_bytes());
            buf.extend_from_slice(b" {}\n");
        }
    }
}

pub fn generate_class_rules_only<'a, I>(buf: &mut Vec<u8>, classes: I, groups: &mut GroupRegistry)
where
    I: IntoIterator<Item = &'a String>,
{
    use cssparser::serialize_identifier;
    let mut escaped = String::with_capacity(64);
    let engine_opt = std::panic::catch_unwind(|| AppState::engine()).ok();
    if let Some(engine) = engine_opt {
        for class in classes {
            println!("[dx-style] generate_class for '{}'", class);
            if groups.is_internal_token(class) {
                println!("[dx-style]  - internal token, skip");
                continue;
            }
            // If this concrete class is a member of a grouped alias, don't emit
            // the individual utility rule. The grouped alias will produce the
            // combined selector and bodies.
            if groups.is_util_member(class) {
                println!("[dx-style]  - is utility member of a group, skipping concrete emission");
                continue;
            }
            if let Some(alias_css) = groups.generate_css_for(class, engine) {
                println!(
                    "[dx-style]  - group generated CSS for '{}': starts with -> {}",
                    class,
                    &alias_css[..alias_css.find('\n').unwrap_or(alias_css.len())]
                );
                buf.extend_from_slice(alias_css.as_bytes());
                if !alias_css.ends_with('\n') {
                    buf.push(b'\n');
                }
                continue;
            }
            println!("[dx-style]  - no group CSS for '{}'", class);
            if let Some(css) = engine.css_for_class(class) {
                buf.extend_from_slice(css.as_bytes());
                if !css.ends_with('\n') {
                    buf.push(b'\n');
                }
            } else {
                buf.push(b'.');
                escaped.clear();
                serialize_identifier(class, &mut escaped).unwrap();
                buf.extend_from_slice(escaped.as_bytes());
                buf.extend_from_slice(b" {}\n");
            }
        }
    } else {
        for class in classes {
            if groups.is_internal_token(class) {
                continue;
            }
            buf.push(b'.');
            escaped.clear();
            serialize_identifier(class, &mut escaped).unwrap();
            buf.extend_from_slice(escaped.as_bytes());
            buf.extend_from_slice(b" {}\n");
        }
    }
}
