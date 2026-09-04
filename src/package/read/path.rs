use super::uri::{validate_package_uri, UnsafePackageUri};
use super::PackageOpenError;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct PackageRoot {
    canonical: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PackagePathResolution {
    Existing(PathBuf),
    Missing(PathBuf),
}

#[derive(Debug)]
pub(crate) enum ResolvePackagePathError {
    Unsafe(UnsafePackageUri),
    Io(PackageOpenError),
}

impl PackageRoot {
    pub(crate) fn open(root: impl AsRef<Path>) -> Result<Self, PackageOpenError> {
        let root = root.as_ref();
        let metadata = fs::metadata(root).map_err(|source| PackageOpenError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        if !metadata.is_dir() {
            return Err(PackageOpenError::RootNotDirectory {
                path: root.to_path_buf(),
            });
        }
        let canonical = fs::canonicalize(root).map_err(|source| PackageOpenError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        Ok(Self { canonical })
    }

    pub(crate) fn resolve(
        &self,
        uri: &str,
    ) -> Result<PackagePathResolution, ResolvePackagePathError> {
        validate_package_uri(uri).map_err(ResolvePackagePathError::Unsafe)?;
        let candidate = self.canonical.join(uri);
        match fs::canonicalize(&candidate) {
            Ok(target) => {
                self.require_contained(uri, &target)?;
                Ok(PackagePathResolution::Existing(target))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let existing_ancestor = nearest_existing_ancestor(&candidate)?;
                self.require_contained(uri, &existing_ancestor)?;
                Ok(PackagePathResolution::Missing(candidate))
            }
            Err(source) => Err(ResolvePackagePathError::Io(PackageOpenError::Io {
                path: candidate,
                source,
            })),
        }
    }

    fn require_contained(&self, uri: &str, target: &Path) -> Result<(), ResolvePackagePathError> {
        if target.starts_with(&self.canonical) {
            Ok(())
        } else {
            Err(unsafe_uri(uri))
        }
    }
}

fn nearest_existing_ancestor(path: &Path) -> Result<PathBuf, ResolvePackagePathError> {
    let mut ancestor = path.parent();
    while let Some(candidate) = ancestor {
        match fs::canonicalize(candidate) {
            Ok(path) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                ancestor = candidate.parent();
            }
            Err(source) => {
                return Err(ResolvePackagePathError::Io(PackageOpenError::Io {
                    path: candidate.to_path_buf(),
                    source,
                }));
            }
        }
    }
    unreachable!("an absolute package candidate always has an existing ancestor")
}

fn unsafe_uri(uri: &str) -> ResolvePackagePathError {
    ResolvePackagePathError::Unsafe(UnsafePackageUri {
        uri: uri.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "ifccad-package-path-{name}-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("create test directory");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn opens_and_canonicalizes_a_directory_root() {
        let root = TestDirectory::new("canonical-root");

        let package_root = PackageRoot::open(root.path()).expect("open package root");

        assert_eq!(
            package_root.canonical,
            fs::canonicalize(root.path()).expect("canonical test root")
        );
    }

    #[test]
    fn rejects_a_file_as_the_package_root() {
        let parent = TestDirectory::new("file-root");
        let file = parent.path().join("package.ifccad");
        fs::write(&file, b"not a directory").expect("write root file");

        let error = PackageRoot::open(&file).expect_err("file root must fail");

        assert!(matches!(
            error,
            PackageOpenError::RootNotDirectory { path } if path == file
        ));
    }

    #[test]
    fn accepts_nested_posix_resource_uris() {
        let root = TestDirectory::new("nested-uri");
        let resources = root.path().join("resources");
        fs::create_dir(&resources).expect("create resources directory");
        let resource = resources.join("drawing.json");
        fs::write(&resource, b"{}").expect("write resource");
        let package_root = PackageRoot::open(root.path()).expect("open package root");

        let resolved = package_root
            .resolve("resources/drawing.json")
            .expect("resolve nested resource");

        assert!(matches!(
            resolved,
            PackagePathResolution::Existing(path)
                if path == fs::canonicalize(resource).expect("canonical resource")
        ));
    }

    #[test]
    fn rejects_unsafe_uri_syntax() {
        let root = TestDirectory::new("unsafe-uri");
        let package_root = PackageRoot::open(root.path()).expect("open package root");

        for uri in [
            "",
            "/absolute.json",
            "C:/absolute.json",
            r"dir\file.json",
            "dir//file.json",
            "./file.json",
            "dir/../file.json",
            "nul\0byte.json",
        ] {
            let error = package_root.resolve(uri).expect_err("unsafe URI must fail");
            assert!(
                matches!(
                    error,
                    ResolvePackagePathError::Unsafe(UnsafePackageUri { uri: ref rejected })
                        if rejected == uri
                ),
                "unexpected result for {uri:?}: {error:?}"
            );
        }
    }

    #[test]
    fn missing_resource_under_a_safe_existing_parent_is_not_a_path_escape() {
        let root = TestDirectory::new("safe-missing");
        let resources = root.path().join("resources");
        fs::create_dir(&resources).expect("create resources directory");
        let package_root = PackageRoot::open(root.path()).expect("open package root");

        let resolved = package_root
            .resolve("resources/missing.json")
            .expect("resolve missing resource");

        assert!(matches!(
            resolved,
            PackagePathResolution::Missing(path)
                if path == fs::canonicalize(resources)
                    .expect("canonical resources")
                    .join("missing.json")
        ));
    }

    #[test]
    fn rejects_existing_resource_reached_through_a_link_outside_the_root() {
        let root = TestDirectory::new("link-root");
        let outside = TestDirectory::new("link-outside");
        fs::write(outside.path().join("outside.json"), b"{}").expect("write outside file");
        let link = root.path().join("escape");
        if let Err(error) = create_directory_link(outside.path(), &link) {
            if cfg!(windows)
                && (error.kind() == io::ErrorKind::PermissionDenied
                    || error.raw_os_error() == Some(1314))
            {
                return;
            }
            panic!("create directory link: {error}");
        }
        let package_root = PackageRoot::open(root.path()).expect("open package root");

        let error = package_root
            .resolve("escape/outside.json")
            .expect_err("linked escape must fail");

        assert!(matches!(
            error,
            ResolvePackagePathError::Unsafe(UnsafePackageUri { uri })
                if uri == "escape/outside.json"
        ));
    }

    #[cfg(windows)]
    fn create_directory_link(original: &Path, link: &Path) -> io::Result<()> {
        std::os::windows::fs::symlink_dir(original, link)
    }

    #[cfg(unix)]
    fn create_directory_link(original: &Path, link: &Path) -> io::Result<()> {
        std::os::unix::fs::symlink(original, link)
    }
}
