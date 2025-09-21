from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, cast


DROP_KEYS = {"mdn_url", "status", "groups", "interfaces"}


def load_json(path: Path) -> Dict[str, Any]:
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def filter_dict(data: Any, keep_keys: Optional[Set[str]] = None) -> Any:
    """Recursively drop unimportant keys and return simplified data.

    - Removes keys in DROP_KEYS anywhere in the structure.
    - Preserves strings, numbers, bools, lists, and dicts.
    """
    if isinstance(data, dict):
        data_dict = cast(Dict[str, Any], data)
        out: Dict[str, Any] = {}
        for k, v in data_dict.items():
            if k in DROP_KEYS:
                continue
            filtered = filter_dict(v, keep_keys)
            # Skip empty dicts
            if isinstance(filtered, dict) and not filtered:
                continue
            out[k] = filtered

        # If keep_keys specified and this dict seems like a property bag (contains any keep_keys),
        # prune to only keep those fields, but always retain nested dicts (e.g., descriptors).
        if keep_keys is not None and any(k in keep_keys for k in out.keys()):
            pruned: Dict[str, Any] = {}
            for k, v in out.items():
                if k in keep_keys or isinstance(v, dict):
                    pruned[k] = v
            out = pruned
        return out
    elif isinstance(data, list):
        data_list = cast(List[Any], data)
        return [filter_dict(x) for x in data_list]
    else:
        return data


def toml_escape_str(value: str) -> str:
    # Use TOML basic strings with escapes; keep it simple and robust
    value = value.replace("\\", "\\\\").replace("\"", "\\\"")
    value = value.replace("\n", "\\n")
    return f'"{value}"'


def toml_key_part(key: str) -> str:
    # Always quote key parts to safely handle characters like @, :, (), spaces, etc.
    part = key.replace("\\", "\\\\").replace("\"", "\\\"")
    return f'"{part}"'


def write_toml_value(key: str, value: Any, lines: List[str], indent: int = 0) -> None:
    pad = " " * indent
    if isinstance(value, str):
        lines.append(f"{pad}{key} = {toml_escape_str(value)}")
    elif isinstance(value, bool):
        lines.append(f"{pad}{key} = {'true' if value else 'false'}")
    elif isinstance(value, (int, float)):
        lines.append(f"{pad}{key} = {value}")
    elif isinstance(value, list):
        value_list = cast(List[Any], value)
        arr_items: list[str] = []
        for item in value_list:
            if isinstance(item, str):
                arr_items.append(toml_escape_str(item))
            elif isinstance(item, bool):
                arr_items.append('true' if item else 'false')
            elif isinstance(item, (int, float)):
                arr_items.append(str(item))
            elif isinstance(item, dict):
                # Inline tables for simple dicts in arrays
                arr_items.append(inline_table(cast(Dict[str, Any], item)))
            else:
                arr_items.append(toml_escape_str(str(item)))
        lines.append(f"{pad}{key} = [ " + ", ".join(arr_items) + " ]")
    elif isinstance(value, dict):
        # Inline table for small dicts (no nested dicts)
        dv = cast(Dict[str, Any], value)
        if not any(isinstance(v, dict) for v in dv.values()):
            lines.append(f"{pad}{key} = {inline_table(dv)}")
        else:
            # Fallback: emit as subtables instead
            for sub_k, sub_v in dv.items():
                sub_key = f"{key}.{toml_key_part(sub_k)}"
                write_toml_section(sub_key, sub_v, lines, header_only=True)
                # When header_only, the section header is written; now write its scalars and nested dicts
                write_toml_section_body(sub_key, sub_v, lines)
    else:
        # Fallback to string
        lines.append(f"{pad}{key} = {toml_escape_str(str(value))}")


def inline_table(d: Dict[str, Any]) -> str:
    parts: list[str] = []
    for k, v in d.items():
        k_part = toml_key_part(k)
        if isinstance(v, str):
            parts.append(f"{k_part} = {toml_escape_str(v)}")
        elif isinstance(v, bool):
            parts.append(f"{k_part} = {'true' if v else 'false'}")
        elif isinstance(v, (int, float)):
            parts.append(f"{k_part} = {v}")
        elif isinstance(v, list):
            # Nested arrays inside inline table (only simple types)
            arr: list[str] = []
            v_list: List[Any] = cast(List[Any], v)
            for item in v_list:
                if isinstance(item, str):
                    arr.append(toml_escape_str(item))
                elif isinstance(item, bool):
                    arr.append('true' if item else 'false')
                elif isinstance(item, (int, float)):
                    arr.append(str(item))
                else:
                    arr.append(toml_escape_str(str(item)))
            parts.append(f"{k_part} = [ " + ", ".join(arr) + " ]")
        else:
            # Avoid nesting dicts inside inline tables; stringify
            parts.append(f"{k_part} = {toml_escape_str(str(v))}")
    return "{ " + ", ".join(parts) + " }"


def write_toml_section(header: str, value: Any, lines: List[str], header_only: bool = False) -> None:
    lines.append("")
    lines.append("[" + header + "]")
    if not header_only:
        write_toml_section_body(header, value, lines)


def write_toml_section_body(header: str, value: Any, lines: List[str]) -> None:
    # Write scalar and list fields first, then expand nested dicts as separate tables
    if not isinstance(value, dict):
        # single value section
        if value is not None:
            write_toml_value("value", value, lines)
        return

    # 1) scalars and arrays
    dv = cast(Dict[str, Any], value)
    for k, v in dv.items():
        if isinstance(v, dict):
            continue
        write_toml_value(toml_key_part(k), v, lines)

    # 2) nested dicts -> nested tables
    for k, v in dv.items():
        if not isinstance(v, dict):
            continue
        sub_header = f"{header}.{toml_key_part(k)}"
        write_toml_section(sub_header, v, lines)


def to_toml(data: Dict[str, Any]) -> str:
    lines: List[str] = []
    # Stable order by key name
    for name in sorted(data.keys()):
        content = data[name]
        section_name = toml_key_part(name)
        write_toml_section(section_name, content, lines)
    result = "\n".join(lines).lstrip() + "\n"
    return result


def convert_file(src: Path, dest: Path) -> None:
    raw = load_json(src)
    filtered = filter_dict(raw, keep_keys=None)
    toml_str = to_toml(filtered)
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(toml_str, encoding="utf-8")


def main(argv: List[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Convert CSS JSON data to filtered TOML files.")
    parser.add_argument("--input", type=Path, default=Path("inspirations/data/css"), help="Input directory containing JSON files")
    parser.add_argument("--output", type=Path, default=Path(".dx/style/css"), help="Output directory for TOML files")
    parser.add_argument("--include-schemas", action="store_true", help="Include *.schema.json files (skipped by default)")
    parser.add_argument(
        "--only-keys",
        type=str,
        default=None,
        help="Comma-separated list of keys to keep within entries (e.g., 'syntax,initial'). Default keeps all except dropped metadata.")
    args = parser.parse_args(argv)

    input_dir: Path = args.input
    output_dir: Path = args.output

    if not input_dir.exists():
        raise SystemExit(f"Input directory not found: {input_dir}")

    keep_keys: Optional[Set[str]] = None
    if args.only_keys:
        keep_keys = {k.strip() for k in args.only_keys.split(",") if k.strip()}

    for src in sorted(input_dir.glob("*.json")):
        if not args.include_schemas and src.name.endswith(".schema.json"):
            continue
        dest = output_dir / (src.stem + ".toml")
        raw = load_json(src)
        filtered = filter_dict(raw, keep_keys=keep_keys)
        toml_str = to_toml(filtered)
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(toml_str, encoding="utf-8")
        print(f"Converted {src} -> {dest}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
