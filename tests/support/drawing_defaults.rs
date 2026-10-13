use ocdraw::ocdraw::{validate_ocdraw_document, OcdrawDocument};

pub fn assert_ocdraw_eq(actual: &OcdrawDocument, expected: &OcdrawDocument) {
    validate_ocdraw_document(actual).unwrap();
    macro_rules! fields {
        ($($field:ident),* $(,)?) => { $(assert_eq!(actual.$field, expected.$field, stringify!($field));)* };
    }
    fields!(
        symbol_sources,
        next_symbol_source_id,
        hatch_entities,
        text_styles,
        text_entities,
        mtext_entities,
        next_text_style_id,
        preservation,
        opaque_entities,
        next_entity_id,
        next_layer_id,
        next_layout_id,
        next_line_pattern_id,
        drawing_id,
        unit,
        plot_style_mode,
        geometric_entities,
        viewports,
        line_patterns,
        line_pattern_scale,
        layers,
        layouts,
        ucs_definitions,
        workspace_state,
        point_display,
        block_definitions,
        scopes,
        view_state,
        model_windows,
        paper_canvases,
        viewport_workspaces
    );
}
