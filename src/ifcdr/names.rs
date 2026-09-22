//! Versioned comparison for the candidate scope/block contract.

mod data {
    include!("unicode_casefold_17.rs");
}

pub(crate) fn name_key(value: &str) -> String {
    let mut folded = String::with_capacity(value.len());
    for scalar in value.chars() {
        match data::CASE_FOLD.binary_search_by_key(&scalar, |(source, _)| *source) {
            Ok(index) => folded.push_str(data::CASE_FOLD[index].1),
            Err(_) => folded.push(scalar),
        }
    }
    folded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_default_folding_preserves_non_case_distinctions() {
        for (a, b) in [("Door", "DOOR"), ("Straße", "STRASSE"), ("σ", "ς")] {
            assert_eq!(name_key(a), name_key(b));
        }
        for (a, b) in [("Door", "Door "), ("é", "e\u{301}"), ("I", "ı")] {
            assert_ne!(name_key(a), name_key(b));
        }
        assert_eq!(name_key("İ"), "i\u{307}");
        assert_eq!(name_key("文字😀"), "文字😀");
    }

    #[test]
    fn every_pinned_full_default_mapping_is_applied() {
        let source = include_str!("../../schemas/ifcdr/unicode/CaseFolding-17.0.0.txt");
        let mut checked = 0;
        for line in source.lines() {
            let content = line.split('#').next().unwrap().trim();
            if content.is_empty() {
                continue;
            }
            let fields: Vec<_> = content.split(';').map(str::trim).collect();
            if !matches!(fields[1], "C" | "F") {
                continue;
            }
            let scalar = char::from_u32(u32::from_str_radix(fields[0], 16).unwrap()).unwrap();
            let expected: String = fields[2]
                .split_whitespace()
                .map(|s| char::from_u32(u32::from_str_radix(s, 16).unwrap()).unwrap())
                .collect();
            assert_eq!(name_key(&scalar.to_string()), expected, "U+{}", fields[0]);
            checked += 1;
        }
        assert!(checked > 1500);
    }
}
