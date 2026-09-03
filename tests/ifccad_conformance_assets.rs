use ifccad::conformance::bundled_conformance_root;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn files_below(root: &Path) -> Vec<PathBuf> {
    fn visit(path: &Path, values: &mut Vec<PathBuf>) {
        let mut entries: Vec<_> = fs::read_dir(path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
            .map(|entry| entry.expect("directory entry"))
            .collect();
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                visit(&path, values);
            } else {
                values.push(path);
            }
        }
    }

    let mut values = Vec::new();
    visit(root, &mut values);
    values
}

fn active_schema_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas")
}

fn frozen_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("conformance")
        .join("1.0.0")
}

fn next_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("conformance")
        .join("next")
}

fn repository_git_dir(repository: &Path) -> Option<PathBuf> {
    let dot_git = repository.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git);
    }
    let pointer = fs::read_to_string(dot_git).ok()?;
    resolve_git_dir_pointer(repository, &pointer)
}

fn resolve_git_dir_pointer(repository: &Path, pointer: &str) -> Option<PathBuf> {
    let path = PathBuf::from(pointer.strip_prefix("gitdir:")?.trim());
    Some(if path.is_absolute() {
        path
    } else {
        repository.join(path)
    })
}

#[test]
fn frozen_reference_files_are_complete_and_valid_json() {
    let expected_root = frozen_root();

    let files = files_below(&expected_root);
    assert_eq!(files.len(), 89, "unexpected frozen reference file count");
    let json_files: Vec<_> = files
        .iter()
        .filter(|path| path.extension().is_some_and(|value| value == "json"))
        .collect();
    let blob_files: Vec<_> = files
        .iter()
        .filter(|path| path.components().any(|part| part.as_os_str() == "blobs"))
        .collect();

    assert_eq!(json_files.len(), 73, "unexpected JSON reference count");
    assert_eq!(blob_files.len(), 14, "unexpected blob reference count");

    for path in json_files {
        let bytes = fs::read(path).expect("read JSON reference");
        assert!(
            !bytes.windows(2).any(|window| window == b"\r\n"),
            "JSON reference does not use LF line endings: {}",
            path.display()
        );
        serde_json::from_slice::<Value>(&bytes)
            .unwrap_or_else(|error| panic!("invalid JSON {}: {error}", path.display()));
    }
}

#[test]
fn bundled_reference_includes_contract_schemas_and_provenance() {
    let root = frozen_root();
    for relative in [
        "manifest.json",
        "manifest-schema-v1.json",
        "schemas/ifcdr/registry-0.5.0.json",
        "schemas/ifcdr/registry-meta-schema-v1.json",
        "schemas/ifcpr/schema-0.1.0.json",
        "PROVENANCE.md",
        "LICENSE",
    ] {
        assert!(root.join(relative).is_file(), "missing {relative}");
    }
}

#[test]
fn active_schemas_start_from_bundled_contract_versions() {
    let active = active_schema_root();
    let bundled = frozen_root().join("schemas");

    for relative in [
        "ifcdr/registry-0.5.0.json",
        "ifcdr/registry-meta-schema-v1.json",
        "ifcpr/schema-0.1.0.json",
    ] {
        assert_eq!(
            fs::read(active.join(relative)).expect("read active schema"),
            fs::read(bundled.join(relative)).expect("read bundled schema"),
            "active schema differs from its 1.0.0 bootstrap source: {relative}"
        );
    }
}

#[test]
fn bundled_reference_points_to_the_next_candidate() {
    assert_eq!(bundled_conformance_root(), next_root());
}

#[test]
fn next_candidate_is_self_contained_and_valid_json() {
    let root = next_root();
    for relative in [
        "manifest.json",
        "manifest-schema-v1.json",
        "schemas/ifcx/ifccad-overlay-0.4.0.json",
        "schemas/ifcx/ifccad-overlay-0.5.0.json",
        "schemas/ifcx/ifccad-drawing-core-0.2.0.json",
        "schemas/ifcdr/registry-0.5.0.json",
        "schemas/ifcdr/registry-meta-schema-v1.json",
        "schemas/ifcpr/schema-0.2.0.json",
        "PROVENANCE.md",
        "LICENSE",
    ] {
        assert!(root.join(relative).is_file(), "missing {relative}");
    }

    for relative in [
        "ifcx/ifccad-overlay-0.4.0.json",
        "ifcx/ifccad-overlay-0.5.0.json",
        "ifcx/ifccad-drawing-core-0.2.0.json",
        "ifcdr/registry-0.5.0.json",
        "ifcdr/registry-meta-schema-v1.json",
        "ifcpr/schema-0.2.0.json",
    ] {
        assert_eq!(
            fs::read(active_schema_root().join(relative)).expect("read active schema"),
            fs::read(root.join("schemas").join(relative)).expect("read candidate schema"),
            "candidate schema differs from active contract: {relative}"
        );
    }

    for path in files_below(&root)
        .into_iter()
        .filter(|path| path.extension().is_some_and(|value| value == "json"))
    {
        let bytes = fs::read(&path).expect("read JSON candidate asset");
        assert!(
            !bytes.windows(2).any(|window| window == b"\r\n"),
            "JSON candidate does not use LF line endings: {}",
            path.display()
        );
        serde_json::from_slice::<Value>(&bytes)
            .unwrap_or_else(|error| panic!("invalid JSON {}: {error}", path.display()));
    }
}

#[test]
fn git_attributes_preserve_reference_bytes() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    let Some(git_dir) = repository_git_dir(repository) else {
        eprintln!("Skipping git attribute check: source is not a Git work tree");
        return;
    };
    let output = Command::new("git")
        .args([
            format!("--git-dir={}", git_dir.display()),
            format!("--work-tree={}", repository.display()),
        ])
        .args([
            "check-attr",
            "text",
            "eol",
            "binary",
            "--",
            "conformance/1.0.0/manifest.json",
            "conformance/1.0.0/packages/valid/source-archive/blobs/fb357f76fddbb1178d0ebb2a6497d9c4929e7b04088729548c03ec578567f044",
            "schemas/ifcdr/registry-0.5.0.json",
        ])
        .output();
    let Ok(output) = output else {
        eprintln!("Skipping git attribute check: git is not available");
        return;
    };

    assert!(output.status.success(), "git check-attr failed");
    let attributes = String::from_utf8(output.stdout).expect("Git attribute output is UTF-8");
    assert!(attributes.contains("manifest.json: text: set"));
    assert!(attributes.contains("manifest.json: eol: lf"));
    assert!(attributes
        .contains("fb357f76fddbb1178d0ebb2a6497d9c4929e7b04088729548c03ec578567f044: binary: set"));
    assert!(attributes
        .contains("fb357f76fddbb1178d0ebb2a6497d9c4929e7b04088729548c03ec578567f044: text: unset"));
    assert!(attributes.contains("registry-0.5.0.json: text: set"));
    assert!(attributes.contains("registry-0.5.0.json: eol: lf"));
}

#[test]
fn resolves_linked_worktree_git_directory_pointer() {
    let repository = Path::new("C:/workspace/ifccad");
    let actual = resolve_git_dir_pointer(
        repository,
        "gitdir: C:/workspace/ifccad-meta/worktrees/fingerprints\n",
    );

    assert_eq!(
        actual,
        Some(PathBuf::from(
            "C:/workspace/ifccad-meta/worktrees/fingerprints"
        ))
    );
}
