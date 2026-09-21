use crate::{bundle, fail, result};
use serde_json::{json, Value};
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
