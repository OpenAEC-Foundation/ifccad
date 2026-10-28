//! Numeric-only CAD mode loss is separate from required band-position proof.
use crate::*;
use ifccad::*;
pub(crate) fn to_cad_mode_loss(
    kind: &IfccadEntityKind,
    id: u64,
    issues: &mut Vec<IfccadDiagnostic>,
) {
    if matches!(kind,IfccadEntityKind::PlanarPolyline{widths:IfccadPlanarPolylineWidths::PerVertex{widths},..} if widths.iter().flatten().all(|v|v.to_bits()==0))
    {
        issues.push(crate::diagnostics::modification("polyline-width-target",format!("entity/{id}/ifccad::geom::planarPolyline/widths"),"numeric-only CAD recapture normalizes an all-positive-zero PerVertex mode to Constant zero"));
    }
}
