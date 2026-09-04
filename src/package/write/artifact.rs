use super::PackageWriteError;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_STAGING_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
/// Fully encoded package files awaiting publication to a directory.
pub struct EncodedPackage {
    files: BTreeMap<String, Vec<u8>>,
}

impl EncodedPackage {
    pub(crate) fn new(files: impl IntoIterator<Item = (String, Vec<u8>)>) -> Self {
        Self {
            files: files.into_iter().collect(),
        }
    }

    /// Iterates over package-relative paths and their encoded bytes.
    pub fn files(&self) -> impl ExactSizeIterator<Item = (&str, &[u8])> {
        self.files
            .iter()
            .map(|(path, bytes)| (path.as_str(), bytes.as_slice()))
    }

    /// Returns one encoded file by its package-relative path.
    pub fn file(&self, path: &str) -> Option<&[u8]> {
        self.files.get(path).map(Vec::as_slice)
    }

    /// Atomically publishes the package as a new directory.
    ///
    /// Existing targets are never overwritten.
    pub fn write_directory(&self, target: impl AsRef<Path>) -> Result<(), PackageWriteError> {
        let target = target.as_ref();
        self.validate_artifact_paths()?;
        if target
            .try_exists()
            .map_err(|source| io_error("inspect target", target, source))?
        {
            return Err(PackageWriteError::TargetExists {
                path: target.to_owned(),
            });
        }

        let file_name = target
            .file_name()
            .filter(|name| !name.is_empty())
            .ok_or_else(|| PackageWriteError::InvalidTarget {
                path: target.to_owned(),
            })?;
        let parent = target
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let parent_metadata = fs::metadata(parent)
            .map_err(|source| io_error("inspect target parent", parent, source))?;
        if !parent_metadata.is_dir() {
            return Err(PackageWriteError::Io {
                operation: "use target parent directory",
                path: parent.to_owned(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotADirectory,
                    "target parent is not a directory",
                ),
            });
        }

        let staging = create_staging_directory(parent, file_name)?;
        let staged = self.stage_files(&staging);
        if let Err(error) = staged {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }

        if target
            .try_exists()
            .map_err(|source| io_error("reinspect target", target, source))?
        {
            let _ = fs::remove_dir_all(&staging);
            return Err(PackageWriteError::TargetExists {
                path: target.to_owned(),
            });
        }
        if let Err(source) = fs::rename(&staging, target) {
            let _ = fs::remove_dir_all(&staging);
            return Err(io_error("publish package directory", target, source));
        }
        Ok(())
    }

    fn validate_artifact_paths(&self) -> Result<(), PackageWriteError> {
        for path in self.files.keys() {
            let parsed = Path::new(path);
            if path.is_empty()
                || parsed.is_absolute()
                || parsed
                    .components()
                    .any(|component| !matches!(component, Component::Normal(_)))
            {
                return Err(PackageWriteError::InvalidArtifactPath { path: path.clone() });
            }
        }
        Ok(())
    }

    fn stage_files(&self, staging: &Path) -> Result<(), PackageWriteError> {
        for (relative, bytes) in &self.files {
            let destination = staging.join(relative);
            let parent =
                destination
                    .parent()
                    .ok_or_else(|| PackageWriteError::InvalidArtifactPath {
                        path: relative.clone(),
                    })?;
            fs::create_dir_all(parent)
                .map_err(|source| io_error("create artifact parent", parent, source))?;
            fs::write(&destination, bytes)
                .map_err(|source| io_error("write artifact", &destination, source))?;
        }
        Ok(())
    }
}

fn create_staging_directory(
    parent: &Path,
    target_name: &std::ffi::OsStr,
) -> Result<PathBuf, PackageWriteError> {
    for _ in 0..1024 {
        let sequence = NEXT_STAGING_ID.fetch_add(1, Ordering::Relaxed);
        let mut staging_name = OsString::from(".");
        staging_name.push(target_name);
        staging_name.push(format!(".ifccad-tmp-{}-{sequence}", std::process::id()));
        let staging = parent.join(staging_name);
        match fs::create_dir(&staging) {
            Ok(()) => return Ok(staging),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(source) => return Err(io_error("create staging directory", &staging, source)),
        }
    }
    Err(PackageWriteError::Io {
        operation: "create unique staging directory",
        path: parent.to_owned(),
        source: std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "could not reserve a unique staging directory",
        ),
    })
}

fn io_error(operation: &'static str, path: &Path, source: std::io::Error) -> PackageWriteError {
    PackageWriteError::Io {
        operation,
        path: path.to_owned(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::super::error::PackageWriteError;
    use super::EncodedPackage;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

    fn temp_root(name: &str) -> PathBuf {
        let nonce = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ifccad-artifact-{name}-{}-{nonce}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn rejects_escaping_artifact_paths_before_writing() {
        let root = temp_root("path");
        let target = root.join("project");
        let artifact = EncodedPackage::new([("../escape.json".to_owned(), vec![1])]);

        assert!(matches!(
            artifact.write_directory(&target),
            Err(PackageWriteError::InvalidArtifactPath { .. })
        ));
        assert!(!target.exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_staging_write_leaves_no_target_or_temporary_sibling() {
        let root = temp_root("rollback");
        let target = root.join("project");
        let artifact = EncodedPackage::new([
            ("resources".to_owned(), vec![1]),
            ("resources/model.ifcdr.json".to_owned(), vec![2]),
        ]);

        assert!(matches!(
            artifact.write_directory(&target),
            Err(PackageWriteError::Io { .. })
        ));
        assert!(!target.exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        fs::remove_dir_all(root).unwrap();
    }
}
