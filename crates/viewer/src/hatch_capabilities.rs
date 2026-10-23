//! Qualification for the selected codec, shared by physical export and preflight.
use crate::cad::VERSIONS;
use crate::hatch_export_facts::{native_facts, HatchExportFacts};
use ifccad_convert::opencadcodec::{CadDocument, DxfVersion, EntityType};
use serde_json::{json, Value};
#[derive(Clone, Copy)]
pub(crate) struct HatchTargetProfile<'a> {
    pub format: &'a str,
    pub version: DxfVersion,
}
pub(crate) struct HatchExportRestriction {
    pub code: &'static str,
    pub location: String,
    pub feature: &'static str,
    pub reason: &'static str,
    pub hard: bool,
}
impl HatchExportRestriction {
    pub fn value(&self, p: HatchTargetProfile<'_>) -> Value {
        json!({"code":self.code,"location":self.location,"feature":self.feature,"format":p.format,"version":p.version.as_str(),"message":self.reason,"policyEffect":if self.hard {"block"}else{"loss"}})
    }
}
impl HatchTargetProfile<'_> {
    pub fn restrictions(self, d: &CadDocument) -> Vec<HatchExportRestriction> {
        let facts = d
            .entities()
            .filter_map(|e| {
                if let EntityType::Hatch(h) = e {
                    Some(HatchExportFacts::cad(h))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        self.for_facts(&facts)
    }
    fn for_facts(self, facts: &[HatchExportFacts]) -> Vec<HatchExportRestriction> {
        let mut out = Vec::new();
        for h in facts {
            let loc = &h.location;
            let mut add = |code, field, feature, reason, hard| {
                out.push(HatchExportRestriction {
                    code,
                    location: format!("{loc}/{field}"),
                    feature,
                    reason,
                    hard,
                })
            };
            if h.unbound {
                add("TARGET_CODEC_HATCH_ASSOCIATION_STATE_LOSS","associationEnabled","associationEnabled","An enabled Hatch association without source objects is cleared by the selected codec",false);
            }
            if self.format == "dxf" && self.version == DxfVersion::AC1015 && h.active_gradient {
                add("HATCH_TARGET_FEATURE_UNSUPPORTED","gradient","gradient","Active DXF gradients require AC1018 or later; the codec's AC1015 tags are not qualified version-conforming output",true);
            } else if self.format == "dwg"
                && self.version == DxfVersion::AC1015
                && (h.active_gradient || h.inactive_gradient)
            {
                add(
                    if h.active_gradient {
                        "HATCH_TARGET_FEATURE_UNSUPPORTED"
                    } else {
                        "TARGET_CODEC_HATCH_GRADIENT_STATE_LOSS"
                    },
                    "gradient",
                    "gradient",
                    "DWG AC1015 does not transport gradient settings",
                    h.active_gradient,
                );
            } else if self.format == "dxf" && h.inactive_gradient {
                add(
                    "TARGET_CODEC_HATCH_GRADIENT_STATE_LOSS",
                    "gradient",
                    "gradient",
                    "DXF output omits inactive gradient settings",
                    false,
                );
            }
            if h.description {
                add(
                    "TARGET_CODEC_HATCH_DESCRIPTION_LOSS",
                    "classicSettings/description",
                    "classicSettings",
                    "CAD output omits the Hatch pattern description",
                    false,
                );
            }
            if h.solid_classic {
                add("TARGET_CODEC_HATCH_CLASSIC_SETTINGS_LOSS","classicSettings","classicSettings","CAD solid-mode output omits classic definition lines, rotation, scale and double settings",false);
            }
            if h.pixel_without_derived {
                add("TARGET_CODEC_HATCH_BOUNDARY_CONSTRUCTION_LOSS","boundaryConstruction/pixelSize","boundaryConstruction","CAD output writes pixel size only when an existing contour has DERIVED provenance",false);
            }
        }
        out
    }
}
pub(crate) fn metadata() -> Value {
    json!({"versions":VERSIONS.map(|v|v.as_str()),"formats":["dxf","dwg"],"qualification":"selectedCodecWriteReadback","sourceGroups":true,"provenance":true,"textIslands":true,"activeGradientMinimum":"AC1018","runtimeReadbackRequired":true})
}
pub(crate) fn preflight(d: &ifccad::IfccadDocument) -> Value {
    // Prepare once, then inspect exactly the mapped fields the writer consumes.
    let mapped = ifccad_convert::ifccad_document_to_cad_document(d, Default::default());
    let fallback = native_facts(d);
    let mut entries = Vec::new();
    for format in ["dxf", "dwg"] {
        for version in VERSIONS {
            let p = HatchTargetProfile { format, version };
            let restrictions = match &mapped {
                Ok(c) => {
                    let mut values = p
                        .restrictions(c.document())
                        .iter()
                        .map(|r| r.value(p))
                        .collect::<Vec<_>>();
                    values.extend(c.diagnostics().iter().filter(|d| d.code.to_ascii_lowercase().contains("hatch")).map(|d| {
                        // This candidate choice was assessed at inspection's
                        // default tolerance, not the user's output settings.
                        let recipe_trial=d.code=="hatch-classic-settings-loss" && d.location.ends_with("/classicSettings/recipe");
                        json!({"code":if recipe_trial {"HATCH_RECIPE_PREFLIGHT_CHECK"} else {d.code},"sourceCode":d.code,"location":d.location,"feature":"hatch","format":format,"version":version.as_str(),"message":d.message,"policyEffect":if recipe_trial {"check"} else {"loss"}})
                    }));
                    values
                }
                Err(_) => {
                    let mut values = p
                        .for_facts(&fallback)
                        .iter()
                        .map(|r| r.value(p))
                        .collect::<Vec<_>>();
                    if !fallback.is_empty() {
                        values.push(json!({"code":"HATCH_PREFLIGHT_UNAVAILABLE","feature":"hatch","format":format,"version":version.as_str(),"message":"Full model preparation was unavailable at the default tolerance; actual export checks the selected output settings","policyEffect":"check"}));
                    }
                    values
                }
            };
            entries.push(
                json!({"format":format,"version":version.as_str(),"restrictions":restrictions}),
            );
        }
    }
    json!(entries)
}
