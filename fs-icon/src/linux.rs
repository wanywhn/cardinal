pub fn icon_of_path_linux(_path: &str) -> Option<Vec<u8>> {
    // Stub implementation - always returns None
    // TODO: Replace with a proper icon extraction library when available
    // Currently returns None as gio dependency has been removed
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
