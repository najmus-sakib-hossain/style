#!/bin/bash

# This script lists the contents of the current directory in a 3-column format:
# Item Name | Size | File Count (folders) or Line Count (files)
# Folders are sorted by file count (most files first)
# Files are sorted by size (biggest first)

echo "🔎 Summary for: $(pwd)"
echo "===================================================================================="

# --- Header ---
printf "%-40s %-15s %-20s\n" "Item (File/Folder)" "Size" "Count"
printf "%-40s %-15s %-20s\n" "----------------------------------------" "---------------" "--------------------"

# --- Function to format size ---
format_size() {
    local bytes=$1
    if (( bytes >= 1073741824 )); then
        echo "$(awk -v b="$bytes" 'BEGIN { printf "%.2f GB", b/1024^3 }')"
    elif (( bytes >= 1048576 )); then
        echo "$(awk -v b="$bytes" 'BEGIN { printf "%.2f MB", b/1024^2 }')"
    elif (( bytes >= 1024 )); then
        echo "$(awk -v b="$bytes" 'BEGIN { printf "%.2f KB", b/1024 }')"
    else
        echo "${bytes} B"
    fi
}

# --- Collect and sort FOLDERS by file count (descending) ---
# Create temporary file for folder data
temp_folders=$(mktemp)
for item in * .[!.]*; do
    if [ -e "$item" ] && [ -d "$item" ]; then
        size_bytes=$(du -sb "$item" 2>/dev/null | awk '{print $1}')
        file_count=$(find "$item" -maxdepth 1 -type f 2>/dev/null | wc -l)
        # Store: file_count|size_bytes|item_name
        echo "${file_count}|${size_bytes}|${item}" >> "$temp_folders"
    fi
done

# Display sorted folders if any exist
if [ -s "$temp_folders" ]; then
    # echo "--- Folders (sorted by file count) ---"
    sort -t'|' -k1 -rn "$temp_folders" | while IFS='|' read -r file_count size_bytes item; do
        size_formatted=$(format_size "$size_bytes")
        printf "📁 %-37s %-15s %d files\n" "$item/" "$size_formatted" "$file_count"
    done
    echo ""
fi
rm -f "$temp_folders"

# --- Collect and sort FILES by size (descending) ---
# Create temporary file for file data
temp_files=$(mktemp)
for item in * .[!.]*; do
    if [ -e "$item" ] && [ -f "$item" ]; then
        size_bytes=$(stat -c%s "$item" 2>/dev/null || stat -f%z "$item" 2>/dev/null)
        
        # Check if it's a text file and get line count
        if file "$item" | grep -q "text\|ASCII\|UTF"; then
            line_count=$(wc -l < "$item" 2>/dev/null)
            # Store: size_bytes|item_name|line_count|type
            echo "${size_bytes}|${item}|${line_count}|text" >> "$temp_files"
        else
            # Store: size_bytes|item_name|0|binary
            echo "${size_bytes}|${item}|0|binary" >> "$temp_files"
        fi
    fi
done

# Display sorted files if any exist
if [ -s "$temp_files" ]; then
    # echo "--- Files (sorted by size) ---"
    sort -t'|' -k1 -rn "$temp_files" | while IFS='|' read -r size_bytes item line_count file_type; do
        size_formatted=$(format_size "$size_bytes")
        if [ "$file_type" = "text" ]; then
            printf "📄 %-37s %-15s %d lines\n" "$item" "$size_formatted" "$line_count"
        else
            printf "📄 %-37s %-15s binary file\n" "$item" "$size_formatted"
        fi
    done
fi
rm -f "$temp_files"

echo "===================================================================================="

# --- Overall Summary ---
echo "### Summary Totals"

# Get total counts for the current directory ONLY
FILE_COUNT=$(find . -maxdepth 1 -type f 2>/dev/null | wc -l)
FOLDER_COUNT=$(find . -maxdepth 1 -mindepth 1 -type d 2>/dev/null | wc -l)

# Get total lines from all text files in current directory
TOTAL_LINES=0
for file in $(find . -maxdepth 1 -type f 2>/dev/null); do
    if file "$file" | grep -q "text\|ASCII\|UTF"; then
        lines=$(wc -l < "$file" 2>/dev/null)
        TOTAL_LINES=$((TOTAL_LINES + lines))
    fi
done

# Get total size
SIZE_BYTES=$(du -sb . 2>/dev/null | awk '{print $1}')
SIZE_FORMATTED=$(format_size "$SIZE_BYTES")

printf "%-20s: %d\n" "Total Files" "$FILE_COUNT"
printf "%-20s: %d\n" "Total Folders" "$FOLDER_COUNT"
printf "%-20s: %d\n" "Total Lines (text)" "$TOTAL_LINES"
printf "%-20s: %s\n" "Total Size" "$SIZE_FORMATTED"
echo "===================================================================================="