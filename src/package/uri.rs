#[derive(Debug, PartialEq, Eq)]
pub(crate) struct UnsafePackageUri {
    pub(crate) uri: String,
}

pub(crate) fn validate_package_uri(uri: &str) -> Result<(), UnsafePackageUri> {
    let segments: Vec<_> = uri.split('/').collect();
    let invalid = uri.is_empty()
        || uri.contains('\0')
        || uri.contains('\\')
        || uri.starts_with('/')
        || segments
            .first()
            .is_some_and(|segment| segment.contains(':'))
        || segments
            .iter()
            .any(|segment| segment.is_empty() || *segment == "." || *segment == "..");

    if invalid {
        Err(UnsafePackageUri {
            uri: uri.to_owned(),
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_relative_posix_package_uris() {
        for uri in [
            "drawing.ifcdr.json",
            "resources/drawing.ifcdr.json",
            "blobs/0123456789abcdef",
        ] {
            assert_eq!(validate_package_uri(uri), Ok(()), "URI {uri:?}");
        }
    }

    #[test]
    fn rejects_unsafe_package_uri_syntax() {
        for uri in [
            "",
            "/absolute.json",
            "C:/absolute.json",
            "scheme:value.json",
            r"dir\file.json",
            "dir//file.json",
            "./file.json",
            "dir/../file.json",
            "nul\0byte.json",
        ] {
            let error = validate_package_uri(uri).expect_err("unsafe URI");
            assert_eq!(error.uri, uri);
        }
    }
}
