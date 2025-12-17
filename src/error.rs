use serde::{Serialize, Serializer};

/// Error types for the liquid-glass plugin
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operation is not supported on this platform
    #[error("Not supported on this platform")]
    UnsupportedPlatform,

    /// The macOS version is not supported
    #[error("macOS version is not supported (requires 26+)")]
    UnsupportedMacOSVersion,

    /// The specified window was not found
    #[error("Window not found: {0}")]
    WindowNotFound(String),

    /// Failed to create glass effect view
    #[error("Failed to create glass effect view")]
    ViewCreationFailed,

    /// Invalid color format
    #[error("Invalid color format: {0}")]
    InvalidColorFormat(String),

    /// Tauri error
    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
}

// Make error serializable for JavaScript
impl Serialize for Error {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

/// Result type for the liquid-glass plugin
pub type Result<T> = std::result::Result<T, Error>;
