# convert_colors.py

import re
import os
from coloraide import Color  # type: ignore
from typing import Any

# --- Configuration ---
# The name of your TypeScript file
FILE_PATH = "presets.ts"
# Precision for the output oklch values. 3 is usually a good balance.
PRECISION = 3

def convert_color_string(color_str: str) -> str:
    """
    Converts a single color string (hsl or hex) to an oklch string.
    If the string is not a valid color, it returns the original string.
    """
    try:
        # Create a Color object from the input string.
        # coloraide automatically detects the format (hex, hsl, etc.).
        color: Any = Color(color_str)  # type: ignore
        
        # Convert to the oklch color space.
        # The .to_string() method is great because it handles NaN hues
        # (for grayscale colors) and formats it nicely.
        # The 'space' keyword argument ensures it's formatted like `oklch(l c h)`.
        oklch_str: str = color.convert("oklch").to_string(precision=PRECISION)  # type: ignore
        
        print(f"Converted '{color_str}' -> '{oklch_str}'")
        return oklch_str  # type: ignore
        
    except Exception:
        # If coloraide fails, it's not a color we can convert.
        # This could be a malformed color or just a random string.
        # We return it unchanged.
        print(f"Skipped '{color_str}' (not a valid color or format).")
        return color_str

def replacement_function(match: Any) -> str:
    """
    This function is called by re.sub for each color found.
    """
    # The matched color string, e.g., "hsl(0 0% 50%)" or "#ff0000"
    original_color: str = match.group(0)
    
    # We only want to convert the color if it's inside quotes
    # The regex looks for quotes, but let's double-check the context if needed.
    # For this script's purpose, a direct conversion is fine.
    
    return convert_color_string(original_color)

def process_file(filepath: str):
    """
    Reads the file, converts colors, and writes it back.
    """
    if not os.path.exists(filepath):
        print(f"Error: File not found at '{filepath}'")
        return

    print(f"--- Processing {filepath} ---")

    # It's always a good idea to back up the original file first.
    backup_path = f"{filepath}.bak"
    try:
        with open(filepath, 'r', encoding='utf-8') as f_in, open(backup_path, 'w', encoding='utf-8') as f_bak:
            original_content = f_in.read()
            f_bak.write(original_content)
        print(f"Backup of original file created at '{backup_path}'")
    except Exception as e:
        print(f"Error creating backup file: {e}")
        return

    # This regex is designed to find HSL and Hex color strings.
    # - `hsl(a?)KATEX_INLINE_OPEN[^)]+KATEX_INLINE_CLOSE`: Matches 'hsl(...)' or 'hsla(...)'. It's non-greedy.
    # - `#([A-Fa-f0-9]{3,4}|[A-Fa-f0-9]{6}|[A-Fa-f0-9]{8})\b`: Matches 3, 4, 6, or 8-digit hex codes.
    #   The `\b` is a word boundary to prevent matching parts of other words (e.g., a URL with a #).
    # The regex specifically avoids matching 'oklch' to prevent re-converting.
    color_regex = re.compile(
        r'hsl(a?)KATEX_INLINE_OPEN[^)]+KATEX_INLINE_CLOSE|#([A-Fa-f0-9]{3,4}|[A-Fa-f0-9]{6}|[A-Fa-f0-9]{8})\b'
    )

    # Use re.sub with our replacement function to process the whole file content.
    print("\n--- Starting Color Conversion ---")
    updated_content = color_regex.sub(replacement_function, original_content)
    print("--- Color Conversion Complete ---\n")

    # Write the modified content back to the original file.
    try:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(updated_content)
        print(f"Successfully updated '{filepath}' with oklch color values.")
    except Exception as e:
        print(f"Error writing updated file: {e}")


if __name__ == "__main__":
    process_file(FILE_PATH)