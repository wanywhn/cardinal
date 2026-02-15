# FS Icon (Cross-Platform Icons & Thumbnails)

This chapter documents the `fs-icon/` crate, which provides cross-platform file/folder icons and thumbnails, supporting both macOS and Linux.

---

## Overview

`fs-icon` exposes cross-platform APIs:
- `icon_of_path(path: &str) -> Option<Vec<u8>>` — best-effort icon as PNG bytes.
  - **macOS**: Tries QuickLook first, then NSWorkspace.
  - **Linux**: Uses system icon theme via GTK or XDG directories.
- `icon_of_path_ns(path: &str) -> Option<Vec<u8>>` — icon from `NSWorkspace::iconForFile` (macOS only).
- `icon_of_path_ql(path: &str) -> Option<Vec<u8>>` — QuickLook-generated thumbnail for image-like files (macOS only).
- `image_dimension(path: &str) -> Option<(f64, f64)>` — lightweight width/height probe via Image I/O (macOS only).

All image data is returned as PNG bytes, ready to be base64-encoded by the Tauri backend.

## Linux Implementation

On Linux, `fs-icon` uses the following approach:

1. **MIME Type Detection**: Uses `mime_guess` to determine the file type based on extension.
2. **Icon Name Mapping**: Maps MIME types to standard icon names (e.g., "folder" for directories, "application-pdf" for PDFs).
3. **Icon Retrieval**: Attempts to retrieve icons via:
   - **GTK Icon Theme**: Uses the system's default icon theme via `gtk::IconTheme`.
   - **XDG Directories**: Falls back to searching standard XDG icon directories for common themes (hicolor, oxygen, gnome) in various sizes and categories (mimetypes, apps, places).
4. **Fallback Strategy**: If no specific icon is found, falls back to generic icons based on file type or directory status.

The Linux implementation supports common icon themes and standard icon sizes (scalable, 256x256, 128x128, 64x64, 48x48, 32x32, 24x24, 16x16).

---

## NSWorkspace-based icons (macOS only)

`icon_of_path_ns` uses the system icon for a file or folder:

1. Build an `NSString` from `path`.
2. Call `NSWorkspace::sharedWorkspace().iconForFile(&path_ns)` to obtain an `NSImage`.
3. Pick an appropriate representation:
   - Prefer a representation near 32×32 for Finder-style icons.
   - Fallback: scale the original image down to a 32×32 bounding box while preserving aspect ratio (via `scale_with_aspect_ratio`).
4. Render into an `NSBitmapImageRep` and encode as PNG.

The function runs inside an autorelease pool to avoid leaking Cocoa objects.

---

## QuickLook thumbnails (macOS only)

`icon_of_path_ql` uses QuickLook to generate thumbnails for image-like content:

1. Use `image_dimension` to discover intrinsic width/height via `CGImageSource`.
2. Compute a scaled target size within a 64×64 thumbnail box, preserving aspect ratio.
3. Build a `QLThumbnailGenerationRequest` with:
   - File URL (`NSURL::fileURLWithPath`),
   - `NSSize` target dimensions,
   - Scale (e.g. `1.0`),
   - `QLThumbnailGenerationRequestRepresentationTypes::LowQualityThumbnail`.
4. Submit the request using `QLThumbnailGenerator::sharedGenerator()` and capture the callback through a `crossbeam_channel`:
   - On success, convert the representation to PNG via `NSBitmapImageRep`.
   - On failure or unsupported file types, return `None`.

QuickLook is generally used for richer, content-aware thumbnails and is tried first in `icon_of_path` on macOS.

---

## Aspect ratio helper

`scale_with_aspect_ratio(width, height, max_width, max_height)`:
- Computes the X/Y scale ratios and picks the smaller one.
- Returns `(scaled_width, scaled_height)` preserving aspect ratio and fitting within the bounding box.

This helper is shared by both NSWorkspace and QuickLook code paths to keep icons visually consistent.

---

## Integration notes

- The Tauri backend uses:
  - `icon_of_path` in `get_nodes_info` to attach icons to rows from the name index (cross-platform).
  - On macOS: `icon_of_path_ql` in the icon viewport worker to load higher-fidelity thumbnails for visible rows.
- UI code only ever sees base64 data URIs (`data:image/png;base64,...`); it is agnostic to the source (NSWorkspace vs QuickLook vs Linux icon theme).
- Platform-specific behavior:
  - **macOS**: Tries QuickLook first, then NSWorkspace icons.
  - **Linux**: Uses system icon theme via GTK/XDG directories.
- Non-image files passed to `icon_of_path_ql` will return `None` on macOS; tests enforce this behavior so callers can fall back gracefully.
