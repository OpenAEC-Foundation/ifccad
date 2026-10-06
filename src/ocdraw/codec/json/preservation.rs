use crate::ocdraw::*;
use serde_json::{json, Value};

macro_rules! tokens {
    ($encode:ident, $decode:ident, $type:ident, {$($variant:ident => $token:literal),+ $(,)?}) => {
        fn $encode(value: $type) -> &'static str { match value { $($type::$variant => $token,)+ } }
        fn $decode(value: &Value) -> Option<$type> { match value.as_str()? { $($token => Some($type::$variant),)+ _ => None } }
    };
}
tokens!(origin, parse_origin, OcdrawPreservationOrigin, {CadDocument => "cadDocument", Dwg => "dwg", Dxf => "dxf"});
tokens!(category, parse_category, OcdrawPreservationCategory, {Entity => "entity", Object => "object", Table => "table", Drawing => "drawing", Layout => "layout", Shared => "shared"});
tokens!(role, parse_role, OcdrawPreservationRole, {Complete => "complete", Supplement => "supplement", Shared => "shared"});
tokens!(representation, parse_representation, OcdrawPreservationRepresentation, {CodecTyped => "codecTyped", CodecOpaque => "codecOpaque"});
tokens!(coverage, parse_coverage, OcdrawPreservationDependencyCoverage, {Qualified => "qualified", Conservative => "conservative", Unknown => "unknown"});
tokens!(kind, parse_kind, OcdrawPreservationPayloadKind, {AdapterSnapshot => "adapterSnapshot", RawDwgRecord => "rawDwgRecord", RawDxfGroups => "rawDxfGroups", RawSection => "rawSection"});

fn target(target: OcdrawPreservationTarget) -> Value {
    use OcdrawPreservationTarget::*;
    let (kind, id) = match target {
        Drawing => return json!({"kind":"drawing"}),
        Entity(id) => ("entity", id),
        Layer(id) => ("layer", u64::from(id)),
        LinePattern(id) => ("linePattern", u64::from(id.0)),
        Layout(id) => ("layout", u64::from(id)),
        Scope(id) => ("scope", u64::from(id)),
        BlockDefinition(id) => ("blockDefinition", u64::from(id)),
        Record(id) => ("record", id.0),
    };
    json!({"kind":kind,"id":id})
}
fn uint32(value: &Value) -> Option<u32> {
    u32::try_from(value.as_u64()?).ok()
}
fn string(value: &Value) -> Option<String> {
    Some(value.as_str()?.to_owned())
}
fn parse_target(value: &Value) -> Option<OcdrawPreservationTarget> {
    use OcdrawPreservationTarget::*;
    Some(match value["kind"].as_str()? {
        "drawing" => Drawing,
        "entity" => Entity(value["id"].as_u64()?),
        "layer" => Layer(uint32(&value["id"])?),
        "linePattern" => LinePattern(LinePatternId(uint32(&value["id"])?)),
        "layout" => Layout(uint32(&value["id"])?),
        "scope" => Scope(uint32(&value["id"])?),
        "blockDefinition" => BlockDefinition(uint32(&value["id"])?),
        "record" => Record(OcdrawPreservationRecordId(value["id"].as_u64()?)),
        _ => return None,
    })
}

pub(crate) fn encode_preservation(p: &OcdrawPreservation) -> Value {
    let sources = p.sources.iter().map(|s| {
        let mut v = json!({"id":s.id,"provider":s.provider,"providerRevision":s.provider_revision,"origin":origin(s.origin)});
        if let Some(version) = &s.source_version { v["sourceVersion"] = json!(version); }
        v
    }).collect::<Vec<_>>();
    let records = p.records.iter().map(|r| {
        let bindings = r.bindings.iter().map(|b| json!({"slot":b.slot,"sourceKey":b.source_key,"target":target(b.target)})).collect::<Vec<_>>();
        let conditions = r.conditions.iter().map(|c| json!({"target":target(c.target),"predicate":c.predicate,"version":c.version,"baseline":super::bytes::encode(&c.baseline)})).collect::<Vec<_>>();
        let mut v = json!({"id":r.id.0,"sourceId":r.source_id,"sourceKey":r.source_key,"category":category(r.category),
            "role":role(r.role),"representation":representation(r.representation),"dependencyCoverage":coverage(r.dependency_coverage),
            "bindings":bindings,"conditions":conditions,"payload":{"schema":r.payload.schema,"version":r.payload.version,"kind":kind(r.payload.kind),"bytes":super::bytes::encode(&r.payload.bytes)}});
        if let Some(subject) = r.subject { v["subject"] = target(subject); }
        v
    }).collect::<Vec<_>>();
    json!({"version":p.version,"nextRecordId":p.next_record_id,"sources":sources,"records":records})
}

fn decode(value: &Value) -> Option<OcdrawPreservation> {
    let sources = value["sources"]
        .as_array()?
        .iter()
        .map(|s| {
            Some(OcdrawPreservationSource {
                id: string(&s["id"])?,
                provider: string(&s["provider"])?,
                provider_revision: string(&s["providerRevision"])?,
                origin: parse_origin(&s["origin"])?,
                source_version: s.get("sourceVersion").map(string).transpose_option()?,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let records = value["records"]
        .as_array()?
        .iter()
        .map(|r| {
            Some(OcdrawPreservationRecord {
                id: OcdrawPreservationRecordId(r["id"].as_u64()?),
                source_id: string(&r["sourceId"])?,
                source_key: string(&r["sourceKey"])?,
                category: parse_category(&r["category"])?,
                role: parse_role(&r["role"])?,
                representation: parse_representation(&r["representation"])?,
                subject: r.get("subject").map(parse_target).transpose_option()?,
                dependency_coverage: parse_coverage(&r["dependencyCoverage"])?,
                bindings: r["bindings"]
                    .as_array()?
                    .iter()
                    .map(|b| {
                        Some(OcdrawPreservationBinding {
                            slot: string(&b["slot"])?,
                            source_key: string(&b["sourceKey"])?,
                            target: parse_target(&b["target"])?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?,
                conditions: r["conditions"]
                    .as_array()?
                    .iter()
                    .map(|c| {
                        Some(OcdrawPreservationCondition {
                            target: parse_target(&c["target"])?,
                            predicate: string(&c["predicate"])?,
                            version: uint32(&c["version"])?,
                            baseline: super::bytes::decode(c["baseline"].as_str()?)?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?,
                payload: OcdrawPreservationPayload {
                    schema: string(&r["payload"]["schema"])?,
                    version: uint32(&r["payload"]["version"])?,
                    kind: parse_kind(&r["payload"]["kind"])?,
                    bytes: super::bytes::decode(r["payload"]["bytes"].as_str()?)?,
                },
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(OcdrawPreservation {
        version: uint32(&value["version"])?,
        next_record_id: value["nextRecordId"].as_u64()?,
        sources,
        records,
    })
}

trait TransposeOption<T> {
    fn transpose_option(self) -> Option<Option<T>>;
}
impl<T> TransposeOption<T> for Option<Option<T>> {
    fn transpose_option(self) -> Option<Option<T>> {
        match self {
            None => Some(None),
            Some(Some(v)) => Some(Some(v)),
            Some(None) => None,
        }
    }
}

pub(crate) fn decode_preservation(
    value: &Value,
) -> Result<Option<OcdrawPreservation>, OcdrawDiagnostic> {
    match value.get("preservation") {
        None => Ok(None),
        Some(value) => decode(value).map(Some).ok_or_else(|| {
            crate::ocdraw::read::diagnostic(
                "PRESERVATION_DECODE",
                "/preservation",
                "preservation requires exact integer backings and canonical byte encodings",
            )
        }),
    }
}
