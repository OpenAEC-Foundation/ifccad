use serde::Serialize;
use sha2::{Digest, Sha256};

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
