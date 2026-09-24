use crate::diagnostic::{
    PackageDiagnostic, PackageDiagnosticContextValue, PackageDiagnosticSeverity,
};
use crate::ifcdr::logical::IfcdrDiagnostic;
use std::collections::BTreeMap;

pub(crate) fn logical_diagnostic(uri: &str, d: IfcdrDiagnostic) -> PackageDiagnostic {
    let base = match d.collection {
        "resource" => "",
        "scope" => "/scopeTable",
        "blockDefinition" => "/blockDefinitionTable",
        "blockInstance" => "/streams/blockInstanceStream",
        "viewport" => "/streams/viewportStream",
        "viewportLayerOverride" => "/streams/viewportLayerOverrideStream",
        "layerBinding" => "/layerBindings",
        "appearanceBinding" => "/appearanceBindings",
        "appearanceOverride" => "/appearanceOverrides",
        "line" => "/streams/lineStream",
        "polyline" => "/streams/polylineStream",
        "planarPolyline" => "/streams/planarPolylineStream",
        "spatialPolyline" => "/streams/spatialPolylineStream",
        "entityOrder" => "/streams/entityOrderStream",
        "entityOrderEntry" => "/streams/entityOrderEntryStream",
        _ => "",
    };
    let location = if matches!(
        d.property,
        "start" | "end" | "row" | "vertices" | "entries" | "base"
    ) {
        base.to_owned()
    } else if d.collection == "resource" && d.property == "nextEntityId" {
        "/header/nextEntityId".into()
    } else if let Some(row) = d.row {
        if matches!(
            d.collection,
            "scope"
                | "blockDefinition"
                | "layerBinding"
                | "appearanceBinding"
                | "appearanceOverride"
        ) {
            format!("{base}/{row}/{}", d.property)
        } else {
            format!("{base}/{}/{row}", d.property)
        }
    } else {
        format!("{base}/{}", d.property)
    };
    PackageDiagnostic {
        category: d.category,
        code: d.code.into(),
        severity: PackageDiagnosticSeverity::Error,
        resource_id: Some(d.resource_id),
        resource_uri: Some(uri.into()),
        location: Some(location),
        context: BTreeMap::from([
            (
                "logicalCollection".into(),
                PackageDiagnosticContextValue::String(d.collection.into()),
            ),
            (
                "logicalProperty".into(),
                PackageDiagnosticContextValue::String(d.property.into()),
            ),
        ]),
        message: d.message,
    }
}
