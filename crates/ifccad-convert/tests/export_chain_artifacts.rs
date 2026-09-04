mod support;

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use support::artifacts::{write_chain_artifacts, write_review_artifacts};
use support::documents::supported_model_space_document;

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

#[test]
fn artifact_writer_creates_the_reviewable_chain_files() {
    let root = temporary_root();
    let artifact_root = root.join("artifacts");
    write_chain_artifacts(&artifact_root, &supported_model_space_document())
        .expect("write chain artifacts");

    assert!(artifact_root.join("source.dxf").is_file());
    assert!(artifact_root.join("ifccad/package.ifcx.json").is_file());
    assert!(artifact_root.join("roundtrip.dxf").is_file());
    assert!(artifact_root.join("diagnostics.txt").is_file());

    fs::remove_dir_all(root).expect("remove temporary artifact root");
}

#[test]
#[ignore = "writes review artifacts below target/chain-artifacts"]
fn write_review_artifacts_for_manual_inspection() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target/chain-artifacts");
    write_review_artifacts(&root).expect("write review artifacts");
    println!(
        "review artifacts: {}",
        root.canonicalize().unwrap().display()
    );
}

fn temporary_root() -> PathBuf {
    let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "ifccad-chain-artifact-test-{}-{id}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("create temporary root");
    root
}
