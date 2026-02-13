use gtk::prelude::IconThemeExt;
use mime_guess;
use std::path::Path;

pub fn icon_of_path_linux(path: &str) -> Option<Vec<u8>> {
    // Initialize GTK once if needed
    gtk::init().ok(); // Ignore errors if already initialized

    // Determine MIME type for the file
    let mime_type = mime_guess::from_path(path).first();
    let icon_name = match mime_type {
        Some(mime) => {
            // Map common MIME types to icon names
            match mime.essence_str() {
                "application/pdf" => "application-pdf",
                "image/jpeg" | "image/jpg" | "image/png" | "image/gif" | "image/bmp" | "image/webp" => "image-x-generic",
                "text/plain" => "text-plain",
                "text/html" => "text-html",
                "audio/mpeg" | "audio/wav" | "audio/flac" | "audio/aac" => "audio-x-generic",
                "video/mp4" | "video/mpeg" | "video/avi" | "video/x-msvideo" => "video-x-generic",
                "application/zip" | "application/x-tar" | "application/x-gzip" | "application/x-bzip2" => "package-x-generic",
                "text/csv" => "x-office-spreadsheet",
                "application/msword" | "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "x-office-document",
                "application/vnd.ms-excel" | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => "x-office-spreadsheet",
                "application/vnd.ms-powerpoint" | "application/vnd.openxmlformats-officedocument.presentationml.presentation" => "x-office-presentation",
                _ => {
                    // If it's a directory, use folder icon
                    if Path::new(path).is_dir() {
                        "folder"
                    } else {
                        // Default file icon
                        "text-x-generic"
                    }
                }
            }
        },
        None => {
            // If it's a directory, use folder icon
            if Path::new(path).is_dir() {
                "folder"
            } else {
                // Default to generic file icon
                "text-x-generic"
            }
        }
    };

    // Try to get the icon from the system
    get_icon_by_name_gio(icon_name, 64).or_else(|| get_icon_by_name_fallback(path))
}

fn get_icon_by_name_gio(icon_name: &str, size: i32) -> Option<Vec<u8>> {
    // Attempt to get the default icon theme without initializing GTK
    // (Initialization might have happened elsewhere or we'll handle the error gracefully)
    let icon_theme = gtk::IconTheme::default()?;

    // Load the icon
    let icon = icon_theme.lookup_icon(
        icon_name,
        size,
        gtk::IconLookupFlags::FORCE_SIZE,
    )?;

    // Get the icon file path
    let filename = icon.filename()?;
    let icon_bytes = std::fs::read(filename.as_path()).ok()?;

    Some(icon_bytes)
}

fn get_icon_by_name_fallback(path: &str) -> Option<Vec<u8>> {
    use xdg;

    // Fallback approach using XDG directories to find icons
    let mime_type = mime_guess::from_path(path).first();
    let icon_name = match mime_type {
        Some(mime) => {
            match mime.essence_str() {
                "application/pdf" => "application-pdf",
                "image/jpeg" | "image/jpg" | "image/png" | "image/gif" | "image/bmp" | "image/webp" => "image-x-generic",
                "text/plain" => "text-plain",
                "text/html" => "text-html",
                "audio/mpeg" | "audio/wav" | "audio/flac" | "audio/aac" => "audio-x-generic",
                "video/mp4" | "video/mpeg" | "video/avi" | "video/x-msvideo" => "video-x-generic",
                "application/zip" | "application/x-tar" | "application/x-gzip" | "application/x-bzip2" => "package-x-generic",
                "text/csv" => "x-office-spreadsheet",
                "application/msword" | "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "x-office-document",
                "application/vnd.ms-excel" | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => "x-office-spreadsheet",
                "application/vnd.ms-powerpoint" | "application/vnd.openxmlformats-officedocument.presentationml.presentation" => "x-office-presentation",
                _ => {
                    if Path::new(path).is_dir() {
                        "folder"
                    } else {
                        "text-x-generic"
                    }
                }
            }
        },
        None => {
            if Path::new(path).is_dir() {
                "folder"
            } else {
                "text-x-generic"
            }
        }
    };

    // Look for icon in standard XDG icon directories
    let xdg_dirs = xdg::BaseDirectories::with_prefix("icons").ok()?;
    let sizes = ["scalable", "256x256", "128x128", "64x64", "48x48", "32x32", "24x24", "16x16"];
    let themes = ["hicolor", "oxygen", "gnome"]; // Common icon themes

    for theme in &themes {
        for size in &sizes {
            if let Some(icon_path) = xdg_dirs.find_data_file(format!("icons/{}/{}/*/{}.png", theme, size, icon_name)) {
                if let Ok(bytes) = std::fs::read(&icon_path) {
                    return Some(bytes);
                }
            }

            // Also try category subdirectories
            let categories = ["mimetypes", "apps", "places", "actions", "devices", "status"];
            for category in &categories {
                if let Some(icon_path) = xdg_dirs.find_data_file(format!("icons/{}/{}/{}/{}.png", theme, size, category, icon_name)) {
                    if let Ok(bytes) = std::fs::read(&icon_path) {
                        return Some(bytes);
                    }
                }
            }
        }
    }

    None
}

pub fn icon_of_path_ql(_path: &str) -> Option<Vec<u8>> {
    // QuickLook is macOS-specific, so return None on Linux
    None
}

pub fn image_dimension(_image_path: &str) -> Option<(f64, f64)> {
    // On Linux, we don't have a direct equivalent for getting image dimensions
    // without loading the full image. For now, return None to maintain compatibility
    // with the macOS implementation that uses this for QuickLook.
    // A more robust solution would use an image processing library like image-rs.
    None
}

pub fn icon_of_path_ns(_path: &str) -> Option<Vec<u8>> {
    // Stub implementation for Linux - not applicable on this platform
    None
}