use opencadcodec::{CadDocument, EntityType, Handle};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Serialize)]
pub struct SourceSelectionManifest {
    pub dimension: String,
    pub entities: Vec<String>,
    pub blocks: Vec<String>,
    pub styles: Vec<String>,
    pub layers: Vec<String>,
    pub pictures: Vec<String>,
    pub owners: BTreeMap<String, String>,
    pub excluded_entities: Vec<String>,
}
fn hex(h: Handle) -> String {
    format!("{:X}", h.value())
}
pub fn select_dimension_source_closure(
    source: &CadDocument,
    handle: Handle,
) -> Result<(CadDocument, SourceSelectionManifest), String> {
    let Some(EntityType::Dimension(dim)) = source.get_entity(handle) else {
        return Err("selected source handle is not a Dimension".into());
    };
    let mut keep = BTreeSet::from([handle]);
    let mut names = BTreeSet::new();
    let mut pending = vec![dim.base().block_name.clone()];
    let parsed =
        cad_dimension::parse_dimension_overrides(dim.base().common.extended_data.records())
            .map_err(|e| e.to_string())?;
    for entry in parsed.entries {
        use cad_dimension::CadDimensionSourceOverride as O;
        if let O::Dimblk(h) | O::Dimblk1(h) | O::Dimblk2(h) = entry {
            if !h.is_null() {
                pending.push(
                    source
                        .block_records
                        .iter()
                        .find(|b| b.handle == h)
                        .ok_or("unresolved Dimension override marker")?
                        .name
                        .clone(),
                );
            }
        }
    }

    // Retain source table records conservatively. Their marker definitions are dependencies too.
    for s in source.dim_styles.iter() {
        for h in [s.dimblk, s.dimblk1, s.dimblk2, s.dimldrblk] {
            if !h.is_null() {
                pending.push(
                    source
                        .block_records
                        .iter()
                        .find(|b| b.handle == h)
                        .ok_or("unresolved style marker")?
                        .name
                        .clone(),
                );
            }
        }
        for n in [&s.dimblk_name, &s.dimblk1_name, &s.dimblk2_name] {
            if !n.is_empty() {
                pending.push(n.clone());
            }
        }
    }
    while let Some(name) = pending.pop() {
        if name.is_empty() || !names.insert(name.clone()) {
            continue;
        }
        let b = source
            .block_records
            .get(&name)
            .ok_or_else(|| format!("unresolved required block {name}"))?;
        for h in b
            .entity_handles
            .iter()
            .copied()
            .chain([b.block_entity_handle, b.block_end_handle])
        {
            if h.is_null() {
                continue;
            }
            let Some(e) = source.get_entity(h) else {
                if b.entity_handles.contains(&h) {
                    return Err(format!("unresolved required source entity {}", hex(h)));
                }
                continue;
            };
            keep.insert(h);
            if let EntityType::Insert(i) = e {
                pending.push(i.block_name.clone());
            }
        }
    }
    for b in source.block_records.iter().filter(|b| {
        b.name.eq_ignore_ascii_case("*Model_Space") || b.name.eq_ignore_ascii_case("*Paper_Space")
    }) {
        for h in [b.block_entity_handle, b.block_end_handle] {
            if source.get_entity(h).is_some() {
                keep.insert(h);
            }
        }
    }
    let mut selected = source.clone();
    let removed_layouts: BTreeSet<_> = selected
        .objects
        .iter()
        .filter_map(|(h, o)| match o {
            opencadcodec::objects::ObjectType::Layout(l) if l.name != "Model" => Some(*h),
            _ => None,
        })
        .collect();
    selected.objects.retain(|h, _| !removed_layouts.contains(h));
    for o in selected.objects.values_mut() {
        match o {
            opencadcodec::objects::ObjectType::Dictionary(d) => {
                d.entries.retain(|(_, h)| !removed_layouts.contains(h))
            }
            opencadcodec::objects::ObjectType::DictionaryWithDefault(d) => {
                d.entries.retain(|(_, h)| !removed_layouts.contains(h))
            }
            _ => {}
        }
    }
    for b in selected.block_records.iter_mut() {
        if removed_layouts.contains(&b.layout) {
            b.layout = Handle::NULL;
        }
    }
    let excluded: Vec<_> = source
        .entities()
        .map(|e| e.common().handle)
        .chain(
            source
                .block_records
                .iter()
                .flat_map(|b| [b.block_entity_handle, b.block_end_handle]),
        )
        .filter(|h| !h.is_null() && !keep.contains(h))
        .collect();
    for h in &excluded {
        selected.remove_entity(*h);
    }
    for b in selected.block_records.iter_mut() {
        b.entity_handles.retain(|h| keep.contains(h));
        b.insert_handles.retain(|h| keep.contains(h));
    }
    let remove: Vec<_> = selected
        .block_records
        .iter()
        .filter(|b| {
            !b.name.eq_ignore_ascii_case("*Model_Space")
                && !b.name.eq_ignore_ascii_case("*Paper_Space")
                && !names.contains(&b.name)
        })
        .map(|b| b.name.clone())
        .collect();
    for n in remove {
        selected.block_records.remove(&n);
    }
    let manifest = SourceSelectionManifest {
        dimension: hex(handle),
        entities: keep.iter().map(|h| hex(*h)).collect(),
        blocks: names
            .iter()
            .map(|n| hex(source.block_records.get(n).unwrap().handle))
            .collect(),
        styles: source
            .dim_styles
            .iter()
            .map(|s| hex(s.handle))
            .chain(source.text_styles.iter().map(|s| hex(s.handle)))
            .collect(),
        layers: source.layers.iter().map(|s| hex(s.handle)).collect(),
        pictures: vec![dim.base().block_name.clone()],
        owners: keep
            .iter()
            .map(|h| {
                (
                    hex(*h),
                    hex(source.get_entity(*h).unwrap().common().owner_handle),
                )
            })
            .collect(),
        excluded_entities: excluded.into_iter().map(hex).collect(),
    };
    Ok((selected, manifest))
}
