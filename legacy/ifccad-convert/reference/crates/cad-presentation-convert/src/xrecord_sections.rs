use opencadcodec::objects::{KnownXRecordKind, ObjectType, XRecordValue};
use opencadcodec::{CadDocument, Handle};
use std::ops::Range;

/// CAD-only section coordinates. Native identity resolution belongs to each adapter.
#[derive(Clone, Debug)]
pub struct CadOverrideSection {
    pub layer: Handle,
    pub viewport: Handle,
    pub record: Handle,
    pub kind: KnownXRecordKind,
    pub dictionary_key: &'static str,
    pub value: XRecordValue,
    pub entries: Range<usize>,
}

/// Strict section grammar prevents the codec's pair scanner from crossing unknown sections.
pub fn cad_override_sections(document: &CadDocument) -> Vec<CadOverrideSection> {
    let mut result = Vec::new();
    for layer in document.layers.iter() {
        for (key, section, code, kind) in [
            (
                "ADSK_XREC_LAYER_COLOR_OVR",
                "{ADSK_LYR_COLOR_OVERRIDE",
                420,
                KnownXRecordKind::LayerViewportColorOverride,
            ),
            (
                "ADSK_XREC_LAYER_ALPHA_OVR",
                "{ADSK_LYR_ALPHA_OVERRIDE",
                440,
                KnownXRecordKind::LayerViewportAlphaOverride,
            ),
            (
                "ADSK_XREC_LAYER_LINETYPE_OVR",
                "{ADSK_LYR_LINETYPE_OVERRIDE",
                343,
                KnownXRecordKind::LayerViewportLinetypeOverride,
            ),
            (
                "ADSK_XREC_LAYER_LINEWT_OVR",
                "{ADSK_LYR_LINEWT_OVERRIDE",
                91,
                KnownXRecordKind::LayerViewportLineweightOverride,
            ),
        ] {
            let Some(record) = document
                .xrecord(layer.handle, key)
                .filter(|r| r.entries_complete)
            else {
                continue;
            };
            let Some(dictionary_handle) = document.extension_dictionary_handle(layer.handle) else {
                continue;
            };
            if record.owner != dictionary_handle
                || document.objects.get(&record.handle)
                    != Some(&ObjectType::XRecord(record.clone()))
            {
                continue;
            }
            if !matches!(document.objects.get(&dictionary_handle), Some(ObjectType::Dictionary(d)) if d.owner == layer.handle)
            {
                continue;
            }
            let mut index = 0;
            while index < record.entries.len() {
                let start = index;
                let entry = &record.entries[index];
                if entry.code != 102
                    || !matches!(&entry.value, XRecordValue::String(s) if s.starts_with('{'))
                {
                    index += 1;
                    continue;
                }
                // Find the balanced end first: never accept a known section nested in unknown data.
                let mut depth = 1;
                index += 1;
                while index < record.entries.len() && depth > 0 {
                    if record.entries[index].code == 102 {
                        match &record.entries[index].value {
                            XRecordValue::String(s) if s.starts_with('{') => depth += 1,
                            XRecordValue::String(s) if s == "}" => depth -= 1,
                            _ => {}
                        }
                    }
                    index += 1;
                }
                if depth != 0
                    || index - start != 4
                    || entry.value != XRecordValue::String(section.into())
                {
                    continue;
                }
                let pair = &record.entries[start + 1..start + 3];
                if pair[0].code != 335 || pair[1].code != code {
                    continue;
                }
                let XRecordValue::Handle(viewport) = pair[0].value else {
                    continue;
                };
                if viewport.is_null() {
                    continue;
                }
                result.push(CadOverrideSection {
                    layer: layer.handle,
                    viewport,
                    record: record.handle,
                    kind,
                    dictionary_key: key,
                    value: pair[1].value.clone(),
                    entries: start..index,
                });
            }
        }
    }
    result
}

/// Only fully consumed containers may be exempted from the source inventory.
/// Mixed records/dictionaries remain residual loss; their supported fields can still be mapped.
pub fn consumed_override_objects(
    document: &CadDocument,
    consumed: &[CadOverrideSection],
) -> std::collections::BTreeSet<Handle> {
    let mut covered = std::collections::BTreeSet::new();
    for handle in consumed.iter().map(|s| s.record) {
        let Some(ObjectType::XRecord(record)) = document.objects.get(&handle) else {
            continue;
        };
        let mut expected = record.clone();
        expected.synchronize_object_references();
        if record.entries_complete
            && record.reactors.is_empty()
            && record.xdictionary_handle.is_none()
            && record.cloning_flags == opencadcodec::objects::XRecord::new().cloning_flags
            && record.object_references == expected.object_references
            && record
                .entries
                .iter()
                .filter(|e| (330..=369).contains(&e.code))
                .zip(&record.object_references)
                .all(|(entry, reference)| {
                    matches!(
                        (entry.code, reference.kind),
                        (335, opencadcodec::objects::ProxyReferenceKind::SoftPointer)
                            | (343, opencadcodec::objects::ProxyReferenceKind::HardPointer)
                    )
                })
            && (0..record.entries.len()).all(|i| {
                consumed
                    .iter()
                    .any(|s| s.record == handle && s.entries.contains(&i))
            })
        {
            covered.insert(handle);
        }
    }
    for (handle, object) in &document.objects {
        if let ObjectType::Dictionary(dictionary) = object {
            if !dictionary.entries.is_empty()
                && dictionary.entries.iter().all(|(key, h)| {
                    covered.contains(h)
                        && consumed.iter().any(|section| {
                            section.record == *h && section.dictionary_key.eq_ignore_ascii_case(key)
                        })
                })
                && dictionary.hard_owner_entries.iter().all(|key| {
                    dictionary
                        .entries
                        .iter()
                        .any(|(entry, _)| entry.eq_ignore_ascii_case(key))
                })
                && dictionary.reactors.is_empty()
                && dictionary.xdictionary_handle.is_none()
                && dictionary.duplicate_cloning == 1
                && document.layers.iter().any(|l| l.handle == dictionary.owner)
                && document.extension_dictionary_handle(dictionary.owner) == Some(*handle)
            {
                covered.insert(*handle);
            }
        }
    }
    covered
}
