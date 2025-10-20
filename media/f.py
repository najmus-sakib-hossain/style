import demjson3  # type: ignore
import os
from typing import Dict, List, Any, cast, Set

# --- Configuration ---
TS_FILE_PATH = "presets.ts"
OUTPUT_CSS_PATH = "themes.css" # Single output file for all themes

def parse_ts_object(ts_content: str) -> Dict[str, Any]:
    """
    Parses a TypeScript file content to extract the main object literal.
    """
    try:
        start_index = ts_content.index('{')
        end_index = ts_content.rindex('}')
        object_str = ts_content[start_index : end_index + 1]
        return demjson3.decode(object_str)  # type: ignore
    except (ValueError, IndexError) as e:
        raise ValueError(f"Could not parse the TypeScript object. Error: {e}")

def generate_css_variables(theme_dict: Dict[str, Any]) -> List[str]:
    """
    Generates a list of indented CSS variable strings from a dictionary.
    """
    css_vars: List[str] = []
    for key, value in theme_dict.items():
        if isinstance(value, str):
            css_vars.append(f"    --{key}: {value};")
    return css_vars

def generate_theme_css(theme_data: Dict[str, Any], theme_name: str) -> str:
    """
    Generates the full CSS string for a specific theme using the
    :root and .dark structure, all within a CSS layer.
    """
    css_lines: List[str] = []
    
    # Start the layer
    css_lines.append(f"@layer {theme_name} {{")
    
    # --- :root block (for light mode and global variables) ---
    css_lines.append("  :root {")

    # Collect keys from light and dark themes to avoid duplication
    excluded_keys: Set[str] = set()
    if 'light' in theme_data and isinstance(theme_data.get('light'), dict):
        excluded_keys.update(theme_data['light'].keys())
    if 'dark' in theme_data and isinstance(theme_data.get('dark'), dict):
        excluded_keys.update(theme_data['dark'].keys())
    excluded_keys.add('name')
    excluded_keys.add('label')
    excluded_keys.add('light')
    excluded_keys.add('dark')
    excluded_keys.add('fonts')

    # Process root-level properties (like radius) first
    for key, value in theme_data.items():
        if isinstance(value, str) and key not in excluded_keys:
            css_lines.append(f"    --{key}: {value};")
        # Special handling for fonts, adding a '--font-' prefix
        elif key == "fonts" and isinstance(value, dict):
            fonts_dict = cast(Dict[str, Any], value)
            for font_key, font_value in fonts_dict.items():
                if font_value: # Only add font if a value is provided
                    css_lines.append(f"    --font-{font_key}: {font_value};")

    # Process light theme properties and add them to the :root block
    if 'light' in theme_data and isinstance(theme_data.get('light'), dict):
        css_lines.extend(generate_css_variables(theme_data['light']))
    
    css_lines.append("  }")
    css_lines.append("") # Add a blank line for readability

    # --- .dark block (for dark mode variables) ---
    if 'dark' in theme_data and isinstance(theme_data.get('dark'), dict):
        css_lines.append("  .dark {")
        for key, value in theme_data['dark'].items():
            if isinstance(value, str):
                light_value = theme_data.get('light', {}).get(key)
                if light_value != value:
                    css_lines.append(f"    --{key}: {value};")
        css_lines.append("  }")

    # Close the layer
    css_lines.append("}")
    
    return "\n".join(css_lines)

def main():
    """Main function to run the script."""
    print(f"Reading presets from '{TS_FILE_PATH}'...")
    
    if not os.path.exists(TS_FILE_PATH):
        print(f"Error: The file '{TS_FILE_PATH}' was not found.")
        return

    try:
        with open(TS_FILE_PATH, 'r', encoding='utf-8') as f:
            ts_content = f.read()
        
        all_presets = parse_ts_object(ts_content)
        
        all_css_blocks: List[str] = []
        print("Generating CSS for all found themes...")

        for theme_name, theme_data in all_presets.items():
            print(f"  - Processing theme: '{theme_name}'")
            theme_css = generate_theme_css(theme_data, theme_name)
            if theme_css:
                all_css_blocks.append(theme_css)
        
        # Join all generated CSS theme blocks, separated by two newlines
        final_css_output = "\n\n".join(all_css_blocks)
        
        with open(OUTPUT_CSS_PATH, 'w', encoding='utf-8') as f:
            f.write(final_css_output)
            
        print(f"\n✅ Successfully created '{OUTPUT_CSS_PATH}' with {len(all_css_blocks)} themes!")

    except Exception as e:
        print(f"An error occurred: {e}")

if __name__ == "__main__":
    main()