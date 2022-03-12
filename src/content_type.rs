use std::path::Path;

/// Determines a specific content type from the extension of the given path.
/// This returns None for any extensions that are handled correctly by default
/// in my testing.
pub fn content_type_from_path(path: &Path) -> Option<String> {
    if let Some(ext) = path.extension() {
        match ext.to_str() {
            Some("svg") => Some("image/svg+xml".to_string()),
            Some("js") => Some("application/javascript".to_string()),
            _ => None,
        }
    } else {
        None
    }
}
