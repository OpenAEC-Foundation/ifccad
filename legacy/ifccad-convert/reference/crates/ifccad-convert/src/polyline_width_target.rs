//! Read-only physical-target qualification of authored native width state.
use crate::*;
use ifccad::*;
use opencadcodec::DxfVersion;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadPolylineWidthCadFormat {
    AsciiDxf,
    BinaryDxf,
    Dwg,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IfccadPolylineWidthCadTarget {
    pub format: IfccadPolylineWidthCadFormat,
    pub version: DxfVersion,
}
fn modification(
    issues: &mut Vec<IfccadDiagnostic>,
    code: &'static str,
    path: impl Into<String>,
    message: impl Into<String>,
) {
    issues.push(crate::diagnostics::modification(code, path, message));
}
fn inspect(
    widths: &IfccadPlanarPolylineWidths,
    path: &str,
    part: Option<&str>,
    target: IfccadPolylineWidthCadTarget,
    issues: &mut Vec<IfccadDiagnostic>,
) {
    use IfccadPlanarPolylineWidths as W;
    if let W::PerVertex { widths } = widths {
        if widths.iter().flatten().all(|v| v.to_bits() == 0) {
            modification(issues,"polyline-width-target",path,"numeric-only CAD recapture normalizes an all-positive-zero PerVertex mode to Constant zero");
        }
    }
    if !widths.is_intrinsic_default() {
        if target.format == IfccadPolylineWidthCadFormat::BinaryDxf
            && target.version < DxfVersion::AC1021
        {
            modification(issues,"polyline-width-target",path,"binary DXF promotes this requested legacy version to AC1024; it is not a qualified requested-version file");
        }
        if let Some(part) = part {
            if target.version != DxfVersion::AC1032 {
                modification(issues,"polyline-width-target",part,"stored Dimension graphics with widths are qualified only for AC1032 in this slice");
            }
        }
    }
    if target.format != IfccadPolylineWidthCadFormat::Dwg {
        return;
    }
    let signed = |value: f64, location: String, issues: &mut Vec<IfccadDiagnostic>| {
        if value.to_bits() == (-0.0f64).to_bits() {
            modification(issues,"polyline-width-target",location,"DWG numeric width encoding normalizes signed zero; native binary64 sign is not retained");
        }
    };
    match widths {
        W::Constant { width } => signed(*width, format!("{path}/width"), issues),
        W::PerVertex { widths } => {
            for (i, pair) in widths.iter().enumerate() {
                for (k, value) in pair.iter().enumerate() {
                    signed(*value, format!("{path}/widths/{i}/{k}"), issues);
                }
            }
        }
        W::Defaulted {
            default_start_width,
            default_end_width,
            overrides,
        } => {
            signed(
                *default_start_width,
                format!("{path}/defaultStartWidth"),
                issues,
            );
            signed(
                *default_end_width,
                format!("{path}/defaultEndWidth"),
                issues,
            );
            for (i, pair) in overrides.iter().enumerate() {
                for (name, value) in [("start", pair.start), ("end", pair.end)] {
                    let location = format!("{path}/overrides/{i}/{name}");
                    if let Some(value) = value {
                        signed(value, location, issues)
                    } else {
                        modification(issues,"polyline-width-dependency",location,"DWG materializes this inherited width as an explicit numeric value; matching effective geometry does not preserve the default dependency");
                    }
                }
            }
        }
    }
}
fn marker<'a>(
    d: &'a IfccadDocument,
    id: u64,
    out: &mut Vec<(u64, &'a IfccadPlanarPolylineWidths)>,
) {
    if let Some(b) = d.blocks.iter().find(|b| b.id == id) {
        for e in b.entities.iter().filter_map(IfccadEntity::as_native) {
            match &e.kind {
                IfccadEntityKind::PlanarPolyline { widths, .. } => out.push((e.id, widths)),
                IfccadEntityKind::BlockInstance { definition_id, .. } => {
                    marker(d, *definition_id, out)
                }
                _ => {}
            }
        }
    }
}
/// Enumerate semantic width losses before a physical writer uses a target. This
/// does not write files or replace actual-header, strict-readback or numeric gates.
pub fn assess_ifccad_polyline_width_target(
    document: &IfccadDocument,
    target: IfccadPolylineWidthCadTarget,
    policy: IfccadLossPolicy,
) -> Result<Vec<IfccadDiagnostic>, IfccadConversionError> {
    validate_ifccad_document(document).map_err(IfccadConversionError::CoreValidation)?;
    if !matches!(
        target.version,
        DxfVersion::AC1015
            | DxfVersion::AC1018
            | DxfVersion::AC1021
            | DxfVersion::AC1024
            | DxfVersion::AC1027
            | DxfVersion::AC1032
    ) {
        return Err(IfccadConversionError::Unsupported(vec![
            crate::diagnostics::diagnostic(
                "polyline-width-target",
                format!("/cad/d{}", document.drawing_id),
                "target version is outside the six offered width profiles",
            ),
        ]));
    }
    let mut issues = Vec::new();
    for entities in std::iter::once(&document.model.entities)
        .chain(document.paper_layouts.iter().map(|p| &p.entities))
        .chain(document.blocks.iter().map(|b| &b.entities))
    {
        for e in entities.iter().filter_map(IfccadEntity::as_native) {
            let node = format!("/cad/d{}/e{}", document.drawing_id, e.id);
            match &e.kind {
                IfccadEntityKind::PlanarPolyline { widths, .. } => inspect(
                    widths,
                    &format!("{node}/ifccad::geom::planarPolyline/widths"),
                    None,
                    target,
                    &mut issues,
                ),
                IfccadEntityKind::Dimension(d) => {
                    if let Some(g) = &d.graphics {
                        for (i, p) in g.parts.iter().enumerate() {
                            let part = format!("{node}/graphics/p{i}");
                            match &p.kind {
                                IfccadDimensionGraphicsKind::PlanarPolyline { widths, .. } => {
                                    inspect(
                                        widths,
                                        &format!("{part}/ifccad::geom::planarPolyline/widths"),
                                        Some(&part),
                                        target,
                                        &mut issues,
                                    )
                                }
                                IfccadDimensionGraphicsKind::BlockInstance {
                                    definition_id,
                                    ..
                                } => {
                                    let mut widths = Vec::new();
                                    marker(document, *definition_id, &mut widths);
                                    for (id, w) in widths {
                                        inspect(w,&format!("/cad/d{}/e{id}/ifccad::geom::planarPolyline/widths",document.drawing_id),Some(&part),target,&mut issues);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut seen = std::collections::HashSet::new();
    issues.retain(|d| seen.insert((d.code, d.location.clone(), d.message.clone())));
    crate::diagnostics::enforce_policy(policy, &issues)?;
    Ok(issues)
}
