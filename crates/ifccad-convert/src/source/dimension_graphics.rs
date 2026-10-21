use opencadcodec::{CadDocument, EntityType, Handle};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Default)]
pub(crate) struct SourcePictureInventory {
    pub links: BTreeMap<Handle, Handle>,
    pub private_roots: BTreeSet<Handle>,
}
fn contains(v: &serde_json::Value, h: Handle) -> bool {
    fn scan(v: &serde_json::Value, h: Handle, reference: bool) -> bool {
        match v {
            serde_json::Value::Number(n) => reference && n.as_u64() == Some(h.value()),
            serde_json::Value::Array(v) => v.iter().any(|v| scan(v, h, reference)),
            serde_json::Value::Object(v) => v.iter().any(|(k, v)| {
                scan(
                    v,
                    h,
                    reference
                        || k.contains("handle")
                        || matches!(
                            k.as_str(),
                            "owner"
                                | "reactors"
                                | "entries"
                                | "members"
                                | "block_record"
                                | "extended_data"
                                | "source_entities"
                                | "dimblk"
                                | "dimblk1"
                                | "dimblk2"
                                | "dimldrblk"
                        ),
                )
            }),
            _ => false,
        }
    }
    scan(v, h, false)
}

impl SourcePictureInventory {
    pub fn new(d: &CadDocument) -> Self {
        let mut out = Self::default();
        for e in d.entities() {
            if let EntityType::Dimension(v) = e {
                if let Some(b) = d.block_records.get(&v.base().block_name) {
                    if !b.is_model_space() && !b.is_paper_space() {
                        out.links.insert(v.base().common.handle, b.handle);
                    }
                }
            }
        }
        for h in out.links.values().copied().collect::<BTreeSet<_>>() {
            let b = d
                .block_records
                .iter()
                .find(|b| b.handle == h)
                .expect("indexed picture");
            if !b.flags.anonymous {
                continue;
            }
            let mut a = b.clone();
            let mut baseline = opencadcodec::BlockRecord::new(&b.name);
            baseline.flags.anonymous = true;
            a.handle = baseline.handle;
            a.block_entity_handle = baseline.block_entity_handle;
            a.block_end_handle = baseline.block_end_handle;
            a.entity_handles.clear();
            a.insert_handles.clear();
            if a.insert_count_bytes.iter().all(|v| *v == 1) {
                a.insert_count_bytes.clear();
            }
            if a != baseline {
                continue;
            }
            let mut used = contains(&serde_json::to_value(&d.header).expect("codec header"), h)
                || d.dim_styles.iter().any(|s| {
                    [s.dimblk, s.dimblk1, s.dimblk2, s.dimldrblk].contains(&h)
                        || [&s.dimblk_name, &s.dimblk1_name, &s.dimblk2_name]
                            .into_iter()
                            .any(|name| name == &b.name)
                        || contains(&serde_json::to_value(s).expect("codec style"), h)
                })
                || d.objects
                    .values()
                    .any(|s| contains(&serde_json::to_value(s).expect("codec object"), h));
            for e in d.entities() {
                if matches!(e,EntityType::Insert(v) if d.block_records.get(&v.block_name).is_some_and(|b|b.handle==h))
                {
                    used = true;
                    break;
                }
                let mut v = serde_json::to_value(e).expect("codec entity");
                fn remove_bookkeeping(v: &mut serde_json::Value) {
                    match v {
                        serde_json::Value::Object(o) => {
                            o.remove("owner_handle");
                            o.remove("handle");
                            o.remove("raw_record");
                            for v in o.values_mut() {
                                remove_bookkeeping(v);
                            }
                        }
                        serde_json::Value::Array(a) => {
                            for v in a {
                                remove_bookkeeping(v);
                            }
                        }
                        _ => (),
                    }
                }
                remove_bookkeeping(&mut v);
                if contains(&v, h) {
                    used = true;
                    break;
                }
            }
            if !used {
                out.private_roots.insert(h);
            }
        }
        out
    }
}
