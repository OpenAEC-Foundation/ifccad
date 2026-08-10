use std::io;
use std::path::PathBuf;

/// Failure that prevents an IFCCAD package from being inspected.
#[derive(Debug, thiserror::Error)]
pub(crate) enum PackageOpenError {
    /// The supplied package root exists but is not a directory.
    #[error("IFCCAD package root is not a directory: {path}")]
    RootNotDirectory { path: PathBuf },

    /// The operating system could not access a required path.
    #[error("failed to access IFCCAD package path {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
