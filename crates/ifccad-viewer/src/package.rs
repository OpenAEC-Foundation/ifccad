use crate::{bundle, fail, result};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;
pub fn inspect_package(root: &Path) -> Value {
    let mut r = result(root, "package");
    match ifccad::package::load_directory_package(root) {
        Ok(outcome) => {
            let strict = outcome.validated_package().is_some();
            r["validation"] = json!({"strictAvailable":strict,"report":outcome.report()});
            if strict {
                match bundle::collect(root) {
                    Ok(b) => r["presentation"] = b,
                    Err(e) => fail(&mut r, "preparing", "PRESENTATION_UNAVAILABLE", e),
                }
            }
        }
        Err(e) => fail(&mut r, "reading", "PACKAGE_OPEN_FAILED", e),
    }
    r
}

/// Inspect package bytes without writing them to a directory.
pub fn inspect_package_files(name: &str, files: &BTreeMap<String, Vec<u8>>) -> Value {
    let mut r = result(Path::new(name), "package");
    r["source"]["name"] = json!(name);
    let outcome = ifccad::package::load_package_files(files);
    let strict = outcome.validated_package().is_some();
    r["validation"] = json!({"strictAvailable":strict,"report":outcome.report()});
    if strict {
        match bundle::collect_files(name, files) {
            Ok(presentation) => r["presentation"] = presentation,
            Err(error) => fail(&mut r, "preparing", "PRESENTATION_UNAVAILABLE", error),
        }
    }
    r
}
