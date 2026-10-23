//! Contract checks for explicit revision-1 mapping declarations.
#[cfg(test)]
pub(super) const MAPPING: &str = include_str!("../../../../schemas/ifccad/cbor-mapping-1.json");
#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::geometry_kernel::CoordinateLengthUnit;
    use crate::{
        dimension::*, geometry_kernel::hatch::*, line_pattern::*, plot_kernel::*, text::*,
        workspace_kernel::*,
    };

    fn check<T: CborValue>(r: &serde_json::Value, name: &str) {
        let ty = &r["types"][name];
        let rows = if ty["kind"] == "record" {
            &ty["fields"]
        } else {
            &ty["variants"]
        };
        let expected = rows
            .as_array()
            .unwrap()
            .iter()
            .map(|v| (v["code"].as_u64().unwrap(), v["name"].as_str().unwrap()))
            .collect::<Vec<_>>();
        assert_eq!(T::registered_codes(), expected, "{name}");
    }
    #[test]
    fn explicit_code_declarations_match_contract() {
        let r: serde_json::Value = serde_json::from_str(super::MAPPING).unwrap();
        check::<IfccadDocument>(&r, "IfccadDocument");
        check::<IfccadFilledQuadrilateral>(&r, "IfccadFilledQuadrilateral");
        check::<IfccadHeader>(&r, "IfccadHeader");
        check::<IfccadIdCounters>(&r, "IfccadIdCounters");
        check::<PlotStyleMode>(&r, "PlotStyleMode");
        check::<IfccadLayout>(&r, "IfccadLayout");
        check::<LayoutOutputSettings>(&r, "LayoutOutputSettings");
        check::<IfccadLayer>(&r, "IfccadLayer");
        check::<IfccadBlockDefinition>(&r, "IfccadBlockDefinition");
        check::<IfccadPaperLayout>(&r, "IfccadPaperLayout");
        check::<IfccadLinePattern>(&r, "IfccadLinePattern");
        check::<IfccadSymbolSource>(&r, "IfccadSymbolSource");
        check::<IfccadTextStyle>(&r, "IfccadTextStyle");
        check::<IfccadDimensionStyle>(&r, "IfccadDimensionStyle");
        check::<IfccadUcsDefinition>(&r, "IfccadUcsDefinition");
        check::<IfccadModelWindow>(&r, "IfccadModelWindow");
        check::<IfccadPointDisplay>(&r, "IfccadPointDisplay");
        check::<IfccadDrawingWorkspaceState>(&r, "IfccadDrawingWorkspaceState");
        check::<IfccadModelViewState>(&r, "IfccadModelViewState");
        check::<IfccadPreservation>(&r, "IfccadPreservation");
        check::<IfccadBoundsQuality>(&r, "IfccadBoundsQuality");
        check::<IfccadBounds3d>(&r, "IfccadBounds3d");
        check::<IfccadEntity>(&r, "IfccadEntity");
        check::<IfccadLayerAppearance>(&r, "IfccadLayerAppearance");
        check::<SymbolSource>(&r, "SymbolSource");
        check::<TextStyleProperties>(&r, "TextStyleProperties");
        check::<DimensionStyleSettings<IfccadTextStyleId, u64, IfccadColor, IfccadLinePatternId>>(
            &r,
            "DimensionStyleSettings<IfccadTextStyleId,u64,IfccadColor,IfccadLinePatternId>",
        );
        check::<WorkspaceView>(&r, "WorkspaceView");
        check::<WorkspaceRenderMode>(&r, "WorkspaceRenderMode");
        check::<WorkspaceGrid>(&r, "WorkspaceGrid");
        check::<WorkspaceSnap>(&r, "WorkspaceSnap");
        check::<IfccadUcsSelection>(&r, "IfccadUcsSelection");
        check::<IfccadPointGlyph>(&r, "IfccadPointGlyph");
        check::<IfccadPointSize>(&r, "IfccadPointSize");
        check::<LayoutMedia>(&r, "LayoutMedia");
        check::<PlotRect>(&r, "PlotRect");
        check::<PlotSettings>(&r, "PlotSettings");
        check::<IfccadNativeEntity>(&r, "IfccadNativeEntity");
        check::<IfccadOpaqueEntity>(&r, "IfccadOpaqueEntity");
        check::<IfccadColor>(&r, "IfccadColor");
        check::<IfccadPaperCanvas>(&r, "IfccadPaperCanvas");
        check::<LinePatternElement<IfccadLinePatternMarker>>(
            &r,
            "LinePatternElement<IfccadLinePatternMarker>",
        );
        check::<FontRequest>(&r, "FontRequest");
        check::<DimensionLineSettings<IfccadColor, IfccadLinePatternId>>(
            &r,
            "DimensionLineSettings<IfccadColor,IfccadLinePatternId>",
        );
        check::<DimensionExtensionLines<IfccadColor, IfccadLinePatternId>>(
            &r,
            "DimensionExtensionLines<IfccadColor,IfccadLinePatternId>",
        );
        check::<DimensionArrowSettings<u64>>(&r, "DimensionArrowSettings<u64>");
        check::<DimensionTextSettings<IfccadTextStyleId, IfccadColor, IfccadLinePatternId>>(
            &r,
            "DimensionTextSettings<IfccadTextStyleId,IfccadColor,IfccadLinePatternId>",
        );
        check::<DimensionFitSettings>(&r, "DimensionFitSettings");
        check::<DimensionNumberSettings>(&r, "DimensionNumberSettings");
        check::<DimensionKind>(&r, "DimensionKind");
        check::<DimensionStyleOverrides<IfccadTextStyleId, u64, IfccadColor, IfccadLinePatternId>>(
            &r,
            "DimensionStyleOverrides<IfccadTextStyleId,u64,IfccadColor,IfccadLinePatternId>",
        );
        check::<WorkspaceProjection>(&r, "WorkspaceProjection");
        check::<WorkspaceClip>(&r, "WorkspaceClip");
        check::<WorkspaceGridStyle>(&r, "WorkspaceGridStyle");
        check::<WorkspaceSnapStyle>(&r, "WorkspaceSnapStyle");
        check::<WorkspaceIsometricPlane>(&r, "WorkspaceIsometricPlane");
        check::<IfccadPreservationSource>(&r, "IfccadPreservationSource");
        check::<IfccadPreservationRecord>(&r, "IfccadPreservationRecord");
        check::<MediaUnit>(&r, "MediaUnit");
        check::<PlotUnit>(&r, "PlotUnit");
        check::<PlotPage>(&r, "PlotPage");
        check::<PlotArea>(&r, "PlotArea");
        check::<PlotMapping>(&r, "PlotMapping");
        check::<PlotOutput>(&r, "PlotOutput");
        check::<PlotOptions>(&r, "PlotOptions");
        check::<IfccadEntityKind>(&r, "IfccadEntityKind");
        check::<IfccadEntityAppearance>(&r, "IfccadEntityAppearance");
        check::<DimensionComponentAppearance<IfccadColor, IfccadLinePatternId>>(
            &r,
            "DimensionComponentAppearance<IfccadColor,IfccadLinePatternId>",
        );
        check::<DimensionExtensionSettings<IfccadColor, IfccadLinePatternId>>(
            &r,
            "DimensionExtensionSettings<IfccadColor,IfccadLinePatternId>",
        );
        check::<DimensionMarker<u64>>(&r, "DimensionMarker<u64>");
        check::<DimensionCenterMark>(&r, "DimensionCenterMark");
        check::<MTextAttachment>(&r, "MTextAttachment");
        check::<DimensionTextOrientation>(&r, "DimensionTextOrientation");
        check::<DimensionTextMovement>(&r, "DimensionTextMovement");
        check::<DimensionLineSpacing>(&r, "DimensionLineSpacing");
        check::<DimensionBackground<IfccadColor>>(&r, "DimensionBackground<IfccadColor>");
        check::<DimensionFitPlacement>(&r, "DimensionFitPlacement");
        check::<DimensionPrimaryNumberSettings>(&r, "DimensionPrimaryNumberSettings");
        check::<DimensionToleranceSettings>(&r, "DimensionToleranceSettings");
        check::<DimensionLineOverrides<IfccadColor, IfccadLinePatternId>>(
            &r,
            "DimensionLineOverrides<IfccadColor,IfccadLinePatternId>",
        );
        check::<DimensionExtensionLinesOverrides<IfccadColor, IfccadLinePatternId>>(
            &r,
            "DimensionExtensionLinesOverrides<IfccadColor,IfccadLinePatternId>",
        );
        check::<DimensionArrowOverrides<u64>>(&r, "DimensionArrowOverrides<u64>");
        check::<DimensionTextOverrides<IfccadTextStyleId, IfccadColor, IfccadLinePatternId>>(
            &r,
            "DimensionTextOverrides<IfccadTextStyleId,IfccadColor,IfccadLinePatternId>",
        );
        check::<DimensionFitOverrides>(&r, "DimensionFitOverrides");
        check::<DimensionNumberOverrides>(&r, "DimensionNumberOverrides");
        check::<WorkspaceClipMode>(&r, "WorkspaceClipMode");
        check::<IfccadPreservationOrigin>(&r, "IfccadPreservationOrigin");
        check::<IfccadPreservationCategory>(&r, "IfccadPreservationCategory");
        check::<IfccadPreservationRole>(&r, "IfccadPreservationRole");
        check::<IfccadPreservationRepresentation>(&r, "IfccadPreservationRepresentation");
        check::<IfccadPreservationDependencyCoverage>(&r, "IfccadPreservationDependencyCoverage");
        check::<IfccadPreservationPayload>(&r, "IfccadPreservationPayload");
        check::<CoordinateLengthUnit>(&r, "CoordinateLengthUnit");
        check::<PlotRotation>(&r, "PlotRotation");
        check::<PlotScale>(&r, "PlotScale");
        check::<PlotPlacement>(&r, "PlotPlacement");
        check::<ShadedPlot>(&r, "ShadedPlot");
        check::<IfccadPlacement>(&r, "IfccadPlacement");
        check::<IfccadLinePatternGeneration>(&r, "IfccadLinePatternGeneration");
        check::<IfccadBlockTransform>(&r, "IfccadBlockTransform");
        check::<IfccadViewport>(&r, "IfccadViewport");
        check::<IfccadText>(&r, "IfccadText");
        check::<IfccadHatch>(&r, "IfccadHatch");
        check::<IfccadMode<IfccadColor>>(&r, "IfccadMode<IfccadColor>");
        check::<IfccadMode<f64>>(&r, "IfccadMode<f64>");
        check::<IfccadMode<IfccadLinePatternId>>(&r, "IfccadMode<IfccadLinePatternId>");
        check::<IfccadOpaqueAppearance>(&r, "IfccadOpaqueAppearance");
        check::<WorkspaceCanvasFrame>(&r, "WorkspaceCanvasFrame");
        check::<IfccadPaperContext>(&r, "IfccadPaperContext");
        check::<IfccadLinePatternMarker>(&r, "IfccadLinePatternMarker");
        check::<DimensionAppearanceChoice<IfccadColor>>(
            &r,
            "DimensionAppearanceChoice<IfccadColor>",
        );
        check::<DimensionAppearanceChoice<f64>>(&r, "DimensionAppearanceChoice<f64>");
        check::<DimensionAppearanceChoice<IfccadLinePatternId>>(
            &r,
            "DimensionAppearanceChoice<IfccadLinePatternId>",
        );
        check::<DimensionExtensionLength>(&r, "DimensionExtensionLength");
        check::<DimensionPadding>(&r, "DimensionPadding");
        check::<DimensionFactorApplication>(&r, "DimensionFactorApplication");
        check::<DimensionDistanceFormat>(&r, "DimensionDistanceFormat");
        check::<DimensionAngleFormat>(&r, "DimensionAngleFormat");
        check::<DimensionZeroSuppression>(&r, "DimensionZeroSuppression");
        check::<DimensionArcSymbol>(&r, "DimensionArcSymbol");
        check::<DimensionAlternateNumberSettings>(&r, "DimensionAlternateNumberSettings");
        check::<DimensionComponentOverrides<IfccadColor, IfccadLinePatternId>>(
            &r,
            "DimensionComponentOverrides<IfccadColor,IfccadLinePatternId>",
        );
        check::<DimensionExtensionOverrides<IfccadColor, IfccadLinePatternId>>(
            &r,
            "DimensionExtensionOverrides<IfccadColor,IfccadLinePatternId>",
        );
        check::<DimensionPrimaryNumberOverrides>(&r, "DimensionPrimaryNumberOverrides");
        check::<IfccadPreservationTarget>(&r, "IfccadPreservationTarget");
        check::<IfccadPreservationBinding>(&r, "IfccadPreservationBinding");
        check::<IfccadPreservationCondition>(&r, "IfccadPreservationCondition");
        check::<IfccadPreservationPayloadKind>(&r, "IfccadPreservationPayloadKind");
        check::<PlotOffsetReference>(&r, "PlotOffsetReference");
        check::<ShadedPlotMode>(&r, "ShadedPlotMode");
        check::<ShadedPlotQuality>(&r, "ShadedPlotQuality");
        check::<IfccadViewportFrame>(&r, "IfccadViewportFrame");
        check::<IfccadViewportView>(&r, "IfccadViewportView");
        check::<IfccadViewportRenderMode>(&r, "IfccadViewportRenderMode");
        check::<IfccadViewportPaperClip>(&r, "IfccadViewportPaperClip");
        check::<TextLayout>(&r, "TextLayout");
        check::<IfccadMText>(&r, "IfccadMText");
        check::<HatchAreaRule>(&r, "HatchAreaRule");
        check::<HatchFill>(&r, "HatchFill");
        check::<IfccadDimension>(&r, "IfccadDimension");
        check::<IfccadDimensionGraphics>(&r, "IfccadDimensionGraphics");
        check::<IfccadDimensionGraphicsPart>(&r, "IfccadDimensionGraphicsPart");
        check::<IfccadDimensionGraphicsKind>(&r, "IfccadDimensionGraphicsKind");
        check::<IfccadDimensionGraphicsOrigin>(&r, "IfccadDimensionGraphicsOrigin");
        check::<IfccadDimensionGraphicsPurpose>(&r, "IfccadDimensionGraphicsPurpose");
        check::<LinePatternTransform>(&r, "LinePatternTransform");
        check::<DimensionAlternateChoice>(&r, "DimensionAlternateChoice");
        check::<ShadedPlotQualityMode>(&r, "ShadedPlotQualityMode");
        check::<IfccadViewportProjection>(&r, "IfccadViewportProjection");
        check::<IfccadViewportDepthClip>(&r, "IfccadViewportDepthClip");
        check::<IfccadViewportLayerOverride>(&r, "IfccadViewportLayerOverride");
        check::<TextHorizontalAlignment>(&r, "TextHorizontalAlignment");
        check::<TextVerticalAlignment>(&r, "TextVerticalAlignment");
        check::<TextRun>(&r, "TextRun");
        check::<MTextFlow>(&r, "MTextFlow");
        check::<CharacterFormat<IfccadColor>>(&r, "CharacterFormat<IfccadColor>");
        check::<ParagraphFormat>(&r, "ParagraphFormat");
        check::<HatchGradient<IfccadColor>>(&r, "HatchGradient<IfccadColor>");
        check::<IfccadHatchLoop>(&r, "IfccadHatchLoop");
        check::<HatchLoopProvenance>(&r, "HatchLoopProvenance");
        check::<HatchClassicSettings>(&r, "HatchClassicSettings");
        check::<HatchPatternDefinitionLine>(&r, "HatchPatternDefinitionLine");
        check::<HatchBoundaryConstruction>(&r, "HatchBoundaryConstruction");
        check::<HatchClassicFill>(&r, "HatchClassicFill");
        check::<HatchCreationType>(&r, "HatchCreationType");
        check::<DimensionFractionLayout>(&r, "DimensionFractionLayout");
        check::<HatchLinePattern>(&r, "HatchLinePattern");
        check::<DimensionDefinition>(&r, "DimensionDefinition");
        check::<DimensionTextPlacement>(&r, "DimensionTextPlacement");
        check::<DimensionLabel>(&r, "DimensionLabel");
        check::<LinePatternRotation>(&r, "LinePatternRotation");
        check::<IfccadViewportWorkspace>(&r, "IfccadViewportWorkspace");
        check::<IfccadViewportClipMode>(&r, "IfccadViewportClipMode");
        check::<MTextColumns>(&r, "MTextColumns");
        check::<MTextBackground<IfccadColor>>(&r, "MTextBackground<IfccadColor>");
        check::<MTextParagraph<IfccadColor>>(&r, "MTextParagraph<IfccadColor>");
        check::<HatchGradientColorMode>(&r, "HatchGradientColorMode");
        check::<HatchBoundary2>(&r, "HatchBoundary2");
        check::<IfccadDimensionAssociation>(&r, "IfccadDimensionAssociation");
        check::<MTextFill<IfccadColor>>(&r, "MTextFill<IfccadColor>");
        check::<TextPadding>(&r, "TextPadding");
        check::<TextColor<IfccadColor>>(&r, "TextColor<IfccadColor>");
        check::<TextHeight>(&r, "TextHeight");
        check::<TextPosition>(&r, "TextPosition");
        check::<TextParagraphAlignment>(&r, "TextParagraphAlignment");
        check::<TextLineSpacing>(&r, "TextLineSpacing");
        check::<HatchGradientType>(&r, "HatchGradientType");
        check::<HatchLineFamily>(&r, "HatchLineFamily");
        check::<DimensionLabelPart>(&r, "DimensionLabelPart");
        check::<DimensionRole>(&r, "DimensionRole");
        check::<DimensionPointSelector>(&r, "DimensionPointSelector");
        check::<MTextColumnHeight>(&r, "MTextColumnHeight");
        check::<TextTabStop>(&r, "TextTabStop");
        check::<MTextInline<IfccadColor>>(&r, "MTextInline<IfccadColor>");
        check::<HatchEdge2>(&r, "HatchEdge2");
        check::<TextTabAlignment>(&r, "TextTabAlignment");
        check::<TextStack<IfccadColor>>(&r, "TextStack<IfccadColor>");
        check::<HatchDash>(&r, "HatchDash");
        check::<TextStackKind>(&r, "TextStackKind");
        check::<TextStackAlignment>(&r, "TextStackAlignment");
    }
}
