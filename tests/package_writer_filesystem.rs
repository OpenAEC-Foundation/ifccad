use ifccad::builder::{
    EncodedIfccadPackage, IfccadPackageBuilder, PackageOptions, PackageWriteError,
};
use ifccad::ifcdr::IfccadLengthUnit;
use ifccad::{PackageId, ResourceId};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

struct TempRoot(PathBuf);

impl TempRoot {
    fn new(name: &str) -> Self {
        let nonce = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ifccad-writer-{name}-{}-{nonce}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn encoded() -> EncodedIfccadPackage {
    IfccadPackageBuilder::new(PackageOptions {
        package_id: PackageId::new("filesystem-test").unwrap(),
        data_version: "1".to_owned(),
        author: "writer tests".to_owned(),
        timestamp: "2026-09-03T12:00:00Z".to_owned(),
        model_layout_name: "Model".to_owned(),
        representation_resource_id: ResourceId::new("modelspace").unwrap(),
        length_unit: IfccadLengthUnit::Metre,
    })
    .unwrap()
    .finish()
    .unwrap()
}

#[test]
fn writes_exact_production_files_to_a_new_directory() {
    let root = TempRoot::new("success");
    let target = root.path().join("project");

    encoded().write_directory(&target).unwrap();

    assert!(target.join("package.ifcx.json").is_file());
    assert!(target
        .join("resources")
        .join("model-space.ifcdr.json")
        .is_file());
    assert!(!target.join("package.json").exists());
}

#[test]
fn refuses_an_existing_target_without_changing_it() {
    let root = TempRoot::new("existing");
    let target = root.path().join("project");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("sentinel.txt"), b"keep me").unwrap();

    assert!(matches!(
        encoded().write_directory(&target),
        Err(PackageWriteError::TargetExists { .. })
    ));
    assert_eq!(fs::read(target.join("sentinel.txt")).unwrap(), b"keep me");
    assert_eq!(fs::read_dir(&target).unwrap().count(), 1);
}
