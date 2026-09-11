use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FileMeasurement {
    pub path: String,
    pub role: String,
    pub bytes: u64,
    pub sha256: String,
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn file(path: &str, role: &str, bytes: &[u8]) -> FileMeasurement {
    FileMeasurement {
        path: path.into(),
        role: role.into(),
        bytes: bytes.len() as u64,
        sha256: digest(bytes),
    }
}

pub fn account(
    files: Vec<FileMeasurement>,
    entities: u64,
    vertices: u64,
    native: bool,
    pure_external_polyline: bool,
) -> Result<Value, String> {
    let mut unique = BTreeMap::new();
    for file in files {
        if let Some(previous) = unique.insert(file.path.clone(), file.clone()) {
            if previous != file {
                return Err(format!("conflicting inventory entry: {}", file.path));
            }
        }
    }
    let mut totals = BTreeMap::from([
        ("ifcx", 0_u64),
        ("ifcdr", 0),
        ("ifcpr", 0),
        ("blob", 0),
        ("cad", 0),
        ("total", 0),
    ]);
    for file in unique.values() {
        if file.role == "total" {
            return Err("total is not a file role".into());
        }
        let subtotal = totals
            .get_mut(file.role.as_str())
            .ok_or("unknown file role")?;
        *subtotal = subtotal
            .checked_add(file.bytes)
            .ok_or("byte count overflow")?;
        let total = totals.get_mut("total").unwrap();
        *total = total.checked_add(file.bytes).ok_or("byte count overflow")?;
    }
    Ok(json!({
        "files": unique.into_values().collect::<Vec<_>>(), "bytes": totals,
        "bytes_per_entity": if native && entities > 0 { Some(totals["total"] as f64 / entities as f64) } else { None },
        "ifcdr_bytes_per_polyline_vertex": if native && pure_external_polyline && vertices > 0 { Some(totals["ifcdr"] as f64 / vertices as f64) } else { None },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_physical_files_once_and_keeps_inline_in_ifcx() {
        let files = vec![
            file("package.ifcx.json", "ifcx", b"inline body"),
            file("blob", "blob", b"shared"),
            file("blob", "blob", b"shared"),
        ];
        let row = account(files, 0, 0, false, false).unwrap();
        assert_eq!(row["bytes"]["total"], 17);
        assert_eq!(row["bytes"]["ifcx"], 11);
        assert_eq!(row["bytes"]["ifcdr"], 0);
        assert_eq!(row["files"].as_array().unwrap().len(), 2);
        assert!(row["bytes_per_entity"].is_null());
        assert!(row["ifcdr_bytes_per_polyline_vertex"].is_null());
    }

    #[test]
    fn inconsistent_duplicate_and_overflow_fail() {
        assert!(account(
            vec![file("x", "blob", b"a"), file("x", "blob", b"ab")],
            1,
            0,
            true,
            false
        )
        .is_err());
        let mut huge = file("x", "blob", b"");
        huge.bytes = u64::MAX;
        assert!(account(vec![huge, file("y", "blob", b"a")], 1, 0, true, false).is_err());
    }

    #[test]
    fn ratios_are_only_defined_for_applicable_rows() {
        let native = account(vec![file("d", "ifcdr", b"12345678")], 2, 4, true, true).unwrap();
        assert_eq!(native["bytes_per_entity"], 4.0);
        assert_eq!(native["ifcdr_bytes_per_polyline_vertex"], 2.0);
        let preservation =
            account(vec![file("d", "ifcdr", b"12345678")], 2, 4, false, true).unwrap();
        assert!(preservation["bytes_per_entity"].is_null());
        assert!(preservation["ifcdr_bytes_per_polyline_vertex"].is_null());
    }
}
