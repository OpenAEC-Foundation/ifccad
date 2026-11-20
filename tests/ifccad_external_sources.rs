use ifccad::*;

#[test]
fn source_location_and_identity_contract() {
    for uri in [
        "../Refs/grond%20plan.dwg",
        "file:///C:/CAD/grondplan.dwg",
        "custom:opaque",
        "grondplan.dwg",
    ] {
        assert!(
            IfccadExternalSourceLocation::UriReference { uri: uri.into() }
                .validate()
                .is_ok(),
            "{uri}"
        );
    }
    for uri in [
        "",
        "../bad%2.dwg",
        "../bad%GG.dwg",
        "C:\\CAD\\a.dwg",
        "a b.dwg",
    ] {
        assert!(
            IfccadExternalSourceLocation::UriReference { uri: uri.into() }
                .validate()
                .is_err(),
            "{uri}"
        );
    }
    for name in ["", ".", "..", "../a.dwg", "C:a.dwg", "a\\b.dwg", "a\0.dwg"] {
        assert!(
            IfccadExternalSourceLocation::SearchFileName {
                file_name: name.into()
            }
            .validate()
            .is_err(),
            "{name}"
        );
    }
    let direct = IfccadExternalSourceLocation::UriReference {
        uri: "grondplan.dwg".into(),
    };
    let search = IfccadExternalSourceLocation::SearchFileName {
        file_name: "grondplan.dwg".into(),
    };
    assert_ne!(direct, search);
    assert!(search.validate().is_ok());
}

#[test]
fn external_source_allocation_keeps_history() {
    let mut ids = IfccadIdCounters {
        next_external_source_id: 9_007_199_254_740_993,
        ..Default::default()
    };
    assert_eq!(
        ids.allocate_external_source_id().unwrap().0,
        9_007_199_254_740_993
    );
    assert_eq!(
        ids.allocate_external_source_id().unwrap().0,
        9_007_199_254_740_994
    );
    ids.next_external_source_id = u64::MAX;
    let before = ids;
    assert_eq!(
        ids.allocate_external_source_id().unwrap_err().domain,
        IfccadIdDomain::ExternalSource
    );
    assert_eq!(ids, before);
}
