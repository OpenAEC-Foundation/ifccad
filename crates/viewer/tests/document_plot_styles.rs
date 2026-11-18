use base64::{engine::general_purpose::STANDARD, Engine};
use ifccad::{
    encode_ifccad_document, load_ifccad_bytes, IfccadColor, IfccadDocument, IfccadEntityKind,
    IfccadMode, IfccadPlotStyleMode,
};
use opencaddocument::{
    entities::Line,
    objects::{Layout, ObjectType, PlaceHolder},
    tables::Layer,
    Color, Document, Dwg, Dxf, DxfVersion, EntityType, Format, Handle,
};
use serde_json::Value;

const PAPER: &str = "Layout1";
const STYLE: &str = "Review";
const STB_LAYER: &str = "Review layer";
const CTB: &str = "review.ctb";
const STB: &str = "review.stb";
const TIMESTAMP: &str = "2026-11-13T12:00:00Z";

#[test]
fn cad_export_rejects_different_tables_with_the_same_file_name() {
    let (bytes, _) = Dwg.write(&stb_drawing()).unwrap();
    let (mut native, _) = import("dwg", &bytes);
    let first_id = native.paper_layouts[0].id;
    let source_id = native.plot_styles.layout_sources[&first_id];
    native
        .plot_styles
        .sources
        .get_mut(&source_id)
        .unwrap()
        .sha256 = Some("11".repeat(32));
    let mut alternate = native.plot_styles.sources[&source_id].clone();
    alternate.file_name = STB.to_ascii_uppercase();
    let alternate_id = native.plot_styles.next_source_id;
    native.plot_styles.next_source_id += 1;
    let mut layout = native.paper_layouts[0].clone();
    layout.id = native.id_counters.allocate_layout_id().unwrap();
    layout.name = "Other sheet".into();
    layout.tab_index += 1;
    layout.entities.clear();
    native
        .plot_styles
        .layout_sources
        .insert(layout.id, alternate_id);
    native.paper_layouts.push(layout);
    for (hash, conflicting) in [("22".repeat(32), true), ("11".repeat(32), false)] {
        alternate.sha256 = Some(hash);
        native
            .plot_styles
            .sources
            .insert(alternate_id, alternate.clone());
        let bytes = encode_ifccad_document(&native).unwrap();
        for format in ["dxf", "dwg"] {
            let result = viewer::document_export_ifccad(
                "two-sheets.ifccad",
                bytes.bytes(),
                format,
                "AC1032",
                "{}",
            );
            if conflicting {
                assert!(result["export"]["download"].is_null(), "{format}");
                let failure = result["failure"].to_string().to_ascii_lowercase();
                for name in ["layout1", "other sheet", STB] {
                    assert!(failure.contains(name), "{failure}");
                }
            } else {
                assert!(
                    result["export"]["download"]["base64"].is_string(),
                    "{result}"
                );
            }
        }
    }
}
const CTB_COLORS: [Color; 7] = [
    Color::Index(7),
    Color::Index(251),
    Color::Index(252),
    Color::Index(253),
    Color::Index(254),
    Color::Index(255),
    Color::Rgb {
        r: 255,
        g: 255,
        b: 255,
    },
];

fn drawing(table: &str) -> Document {
    let mut document = Document::with_version(DxfVersion::AC1032);
    let layout = document
        .objects
        .values_mut()
        .find_map(|object| match object {
            ObjectType::Layout(layout) if layout.name == PAPER => Some(layout),
            _ => None,
        })
        .expect("default Paper layout");
    layout.paper_width = 210.0;
    layout.paper_height = 297.0;
    layout.plot_paper_units = 1;
    layout.plot_type = 5;
    layout.plot_scale_numerator = 1.0;
    layout.plot_scale_denominator = 1.0;
    layout.plot_flags.use_standard_scale = false;
    layout.plot_flags.plot_plot_styles = true;
    layout.plot_style_sheet = table.into();
    document
}

fn ctb_drawing() -> Document {
    let mut document = drawing(CTB);
    document.header.plotstyle_mode = true;
    document.layers.get_mut("0").unwrap().color = Color::Index(252);
    for (row, color) in CTB_COLORS.into_iter().enumerate() {
        let y = 10.0 * (row + 1) as f64;
        let mut line = Line::from_coords(10.0, y, 0.0, 30.0, y, 0.0);
        line.common.color = color;
        document
            .add_paper_space_entity(EntityType::Line(line))
            .unwrap();
    }
    document
}

fn stb_drawing() -> Document {
    let mut document = drawing(STB);
    document.header.plotstyle_mode = false;
    let dictionary_handle = document.header.acad_plotstylename_dict_handle;
    let mut style = PlaceHolder::new();
    style.handle = document.allocate_handle();
    style.owner = dictionary_handle;
    let style_handle = style.handle;
    document
        .objects
        .insert(style_handle, ObjectType::PlaceHolder(style));
    let Some(ObjectType::DictionaryWithDefault(dictionary)) =
        document.objects.get_mut(&dictionary_handle)
    else {
        panic!("default CAD plot-style dictionary");
    };
    dictionary.add_entry(STYLE, style_handle);
    let mut layer = Layer::new(STB_LAYER);
    layer.handle = document.allocate_handle();
    layer.plotstyle_handle = style_handle;
    document.layers.add(layer).unwrap();
    let mut line = Line::from_coords(10.0, 10.0, 0.0, 30.0, 10.0, 0.0);
    line.common.layer = STB_LAYER.into();
    line.common.plotstyle_flags = 0b11;
    line.common.plotstyle_handle = Some(style_handle);
    document
        .add_paper_space_entity(EntityType::Line(line))
        .unwrap();
    document
}

fn cad_layout(document: &Document) -> &Layout {
    document
        .objects
        .values()
        .find_map(|object| match object {
            ObjectType::Layout(layout) if layout.name == PAPER => Some(layout),
            _ => None,
        })
        .expect("Paper layout")
}

fn cad_lines(document: &Document) -> Vec<&Line> {
    let owner = cad_layout(document).block_record;
    let mut lines: Vec<_> = document
        .entities()
        .filter_map(|entity| match entity {
            EntityType::Line(line) if line.common.owner_handle == owner => Some(line),
            _ => None,
        })
        .collect();
    lines.sort_by(|left, right| left.start.y.total_cmp(&right.start.y));
    lines
}

fn cad_colors(document: &Document) -> Vec<Color> {
    cad_lines(document)
        .iter()
        .map(|line| line.common.color)
        .collect()
}

fn assigned_style<'a>(document: &'a Document, line: &Line) -> Option<&'a str> {
    if line.common.plotstyle_flags != 0b11 {
        return None;
    }
    let handle = line.common.plotstyle_handle?;
    assert!(matches!(
        document.objects.get(&handle),
        Some(ObjectType::PlaceHolder(_))
    ));
    let dictionary = document
        .named_objects_dictionary()?
        .get("ACAD_PLOTSTYLENAME")?;
    let entries = match document.objects.get(&dictionary)? {
        ObjectType::Dictionary(dictionary) => &dictionary.entries,
        ObjectType::DictionaryWithDefault(dictionary) => &dictionary.entries,
        other => panic!("plot-style dictionary has unexpected kind: {other:?}"),
    };
    entries
        .iter()
        .find_map(|(name, candidate)| (*candidate == handle).then_some(name.as_str()))
}

fn assert_cad_layout(document: &Document, table: &str) {
    assert_eq!(document.version, DxfVersion::AC1032);
    let layout = cad_layout(document);
    assert_eq!(layout.plot_style_sheet, table);
    assert!(layout.plot_flags.plot_plot_styles);
}

fn downloaded(result: &Value) -> Vec<u8> {
    assert!(result["failure"].is_null(), "{result}");
    assert_eq!(result["conversion"]["backend"], "opencaddocument");
    assert_eq!(result["validation"]["strictAvailable"], true, "{result}");
    STANDARD
        .decode(result["export"]["download"]["base64"].as_str().unwrap())
        .unwrap()
}

fn import(extension: &str, source: &[u8]) -> (IfccadDocument, Value) {
    let result = viewer::document_cad_to_ifccad(
        &format!("plot-style.{extension}"),
        extension,
        source,
        TIMESTAMP,
        "{}",
    );
    let native = load_ifccad_bytes(&downloaded(&result), Default::default())
        .unwrap()
        .into_document();
    (native, result["conversion"]["diagnostics"].clone())
}

fn export(native: &IfccadDocument, extension: &str, codec: &dyn Format) -> (Document, Value) {
    let bytes = encode_ifccad_document(native).unwrap();
    let result =
        viewer::document_export_ifccad("plot-style.ifcx", bytes.bytes(), extension, "AC1032", "{}");
    let returned = codec.read(&downloaded(&result)).unwrap().0;
    assert_eq!(result["export"]["fileCheck"]["strictNativeReadback"], true);
    (returned, result["export"]["diagnostics"].clone())
}

fn assert_native_layout(native: &IfccadDocument, table: &str) {
    assert_eq!(native.paper_layouts.len(), 1);
    let layout = &native.paper_layouts[0];
    assert_eq!(layout.name, PAPER);
    let output = &layout.settings.plot_settings.as_ref().unwrap().output;
    let source_id = native.plot_styles.layout_sources[&layout.id];
    assert_eq!(native.plot_styles.sources[&source_id].file_name, table);
    assert!(output.apply_plot_styles);
}

fn assert_no_loss_reported(diagnostics: &Value, stage: &str) {
    let diagnostics = diagnostics.as_array().expect("Document diagnostics");
    assert!(
        diagnostics.iter().all(|entry| entry["action"] != "loss"),
        "{stage}: {diagnostics:?}"
    );
}

fn assert_located_loss(diagnostics: &Value, message: &str, handle: Handle, stage: &str) {
    let handle = format!("{:X}", handle.value());
    let diagnostics = diagnostics.as_array().expect("Document diagnostics");
    assert!(
        diagnostics.iter().any(|entry| {
            entry["action"] == "loss"
                && entry["handle"] == handle
                && entry["message"].as_str().unwrap().contains(message)
        }),
        "{stage}: expected {message:?} on {handle}, got {diagnostics:?}"
    );
}

fn native_color(color: Color) -> IfccadMode<IfccadColor> {
    let value = match color {
        Color::Index(7) => IfccadColor::Foreground,
        Color::Index(index) => {
            let (r, g, b) = color.rgb().unwrap();
            IfccadColor::Concrete {
                rgb: [r, g, b],
                indexed: Some(("ACI".into(), index.into())),
                named: None,
            }
        }
        Color::Rgb { r, g, b } => IfccadColor::rgb(r, g, b),
        _ => panic!("unexpected test color: {color:?}"),
    };
    IfccadMode::Explicit(value)
}

#[test]
fn ctb_color_methods_and_layout_survive_physical_cad() {
    for (extension, codec) in [("dxf", &Dxf as &dyn Format), ("dwg", &Dwg as &dyn Format)] {
        let (source_bytes, _) = codec.write(&ctb_drawing()).unwrap();
        let (source, _) = codec.read(&source_bytes).unwrap();
        assert_cad_layout(&source, CTB);
        assert!(source.header.plotstyle_mode);
        assert_eq!(cad_colors(&source), CTB_COLORS, "{extension} source");

        let (mut native, diagnostics) = import(extension, &source_bytes);
        assert_native_layout(&native, CTB);
        assert_eq!(native.plot_style_mode, IfccadPlotStyleMode::ColorDependent);
        assert_no_loss_reported(&diagnostics, &format!("{extension} CTB import"));
        let lines = &mut native.paper_layouts[0].entities;
        lines.sort_by(|left, right| {
            let y = |entity: &ifccad::IfccadEntity| match &entity.as_native().unwrap().kind {
                IfccadEntityKind::LineSegment { start, .. } => start[1],
                _ => panic!("expected a Paper line"),
            };
            y(left).total_cmp(&y(right))
        });
        assert_eq!(
            lines
                .iter()
                .map(|line| line.as_native().unwrap().appearance.color.clone())
                .collect::<Vec<_>>(),
            CTB_COLORS.map(native_color),
            "{extension} native colors",
        );
        let (returned, diagnostics) = export(&native, extension, codec);
        assert_cad_layout(&returned, CTB);
        assert!(returned.header.plotstyle_mode);
        assert_eq!(returned.layers.get("0").unwrap().color, Color::Index(252));
        assert_no_loss_reported(&diagnostics, &format!("{extension} CTB export"));
        assert_eq!(
            cad_colors(&returned),
            CTB_COLORS,
            "{extension} returned colors"
        );

        let color = &mut native.paper_layouts[0].entities[1]
            .as_native_mut()
            .unwrap()
            .appearance
            .color;
        let IfccadMode::Explicit(IfccadColor::Concrete { rgb, .. }) = color else {
            panic!("expected the concrete ACI 251 line");
        };
        *rgb = [254, 253, 252];
        let layer = native
            .layers
            .iter_mut()
            .find(|layer| layer.name == "0")
            .unwrap();
        layer.appearance.color = IfccadColor::rgb(253, 252, 251)
            .with_indexed("ACI", 252)
            .unwrap();
        let (edited, diagnostics) = export(&native, extension, codec);
        assert_cad_layout(&edited, CTB);
        assert_eq!(
            cad_colors(&edited),
            CTB_COLORS,
            "{extension} after fallback edit"
        );
        let layer = edited.layers.get("0").unwrap();
        assert_eq!(layer.color, Color::Index(252));
        assert_located_loss(
            &diagnostics,
            "RGB fallback for ACI 251",
            cad_lines(&edited)[1].common.handle,
            extension,
        );
        assert_located_loss(
            &diagnostics,
            "RGB fallback for ACI 252",
            layer.handle,
            extension,
        );
    }
}

#[test]
fn stb_named_mode_and_assignments_survive_physical_cad() {
    for (extension, codec) in [("dxf", &Dxf as &dyn Format), ("dwg", &Dwg as &dyn Format)] {
        let (source_bytes, _) = codec.write(&stb_drawing()).unwrap();
        let (source, _) = codec.read(&source_bytes).unwrap();
        assert_cad_layout(&source, STB);
        assert!(!source.header.plotstyle_mode, "{extension} source mode");
        let [line] = cad_lines(&source)
            .try_into()
            .expect("one source Paper line");
        assert_eq!(
            assigned_style(&source, line),
            Some(STYLE),
            "{extension} source assignment"
        );
        let layer = source.layers.get(STB_LAYER).unwrap();
        assert_eq!(Some(layer.plotstyle_handle), line.common.plotstyle_handle);

        let (native, import_diagnostics) = import(extension, &source_bytes);
        assert_native_layout(&native, STB);
        assert_eq!(
            native.plot_style_mode,
            IfccadPlotStyleMode::Named,
            "{extension} imported mode"
        );
        let style = ifccad::IfccadPlotStyle::Named { name: STYLE.into() };
        let layer = native
            .layers
            .iter()
            .find(|layer| layer.name == STB_LAYER)
            .unwrap();
        let object_id = native.paper_layouts[0].entities[0].id();
        assert_eq!(native.plot_styles.layer_assignments[&layer.id], style);
        assert_eq!(native.plot_styles.entity_assignments[&object_id], style);
        assert_no_loss_reported(&import_diagnostics, &format!("{extension} STB import"));
        let (returned, export_diagnostics) = export(&native, extension, codec);
        assert_cad_layout(&returned, STB);
        assert!(!returned.header.plotstyle_mode, "{extension} returned mode");
        let [line] = cad_lines(&returned)
            .try_into()
            .expect("one returned Paper line");
        assert_eq!(
            assigned_style(&returned, line),
            Some(STYLE),
            "{extension} returned assignment"
        );
        let returned_layer = returned.layers.get(STB_LAYER).unwrap();
        assert_eq!(
            Some(returned_layer.plotstyle_handle),
            line.common.plotstyle_handle
        );
        assert_no_loss_reported(&export_diagnostics, &format!("{extension} STB export"));
    }
}

#[test]
fn disabled_table_binding_survives_and_cad_reports_lost_hash() {
    for (extension, codec) in [("dxf", &Dxf as &dyn Format), ("dwg", &Dwg as &dyn Format)] {
        let mut source = ctb_drawing();
        let layout = source
            .objects
            .values_mut()
            .find_map(|object| match object {
                ObjectType::Layout(layout) if layout.name == PAPER => Some(layout),
                _ => None,
            })
            .unwrap();
        layout.plot_flags.plot_plot_styles = false;
        let (source_bytes, _) = codec.write(&source).unwrap();
        let (mut native, diagnostics) = import(extension, &source_bytes);
        assert_no_loss_reported(&diagnostics, extension);
        let layout = &native.paper_layouts[0];
        assert!(
            !layout
                .settings
                .plot_settings
                .as_ref()
                .unwrap()
                .output
                .apply_plot_styles
        );
        let source_id = native.plot_styles.layout_sources[&layout.id];
        let table = native.plot_styles.sources.get_mut(&source_id).unwrap();
        assert_eq!(table.file_name, CTB);
        table.sha256 = Some("0123456789abcdef".repeat(4));
        let (returned, diagnostics) = export(&native, extension, codec);
        let layout = cad_layout(&returned);
        assert_eq!(layout.plot_style_sheet, CTB);
        assert!(!layout.plot_flags.plot_plot_styles);
        assert_located_loss(
            &diagnostics,
            "without its SHA-256",
            layout.handle,
            extension,
        );
    }
}

#[test]
fn external_plot_probe_dxf_preserves_explicit_by_block() {
    let mut source = drawing(STB);
    source.header.plotstyle_mode = false;
    let mut line = Line::from_coords(10.0, 10.0, 0.0, 30.0, 10.0, 0.0);
    line.common.plotstyle_flags = 1;
    source
        .add_paper_space_entity(EntityType::Line(line))
        .unwrap();
    let (bytes, _) = Dxf.write(&source).unwrap();
    let (returned, _) = Dxf.read(&bytes).unwrap();
    let [line] = cad_lines(&returned).try_into().expect("one Paper line");
    assert_eq!(line.common.plotstyle_flags, 1, "DXF changed ByBlock");
    let (native, diagnostics) = import("dxf", &bytes);
    assert_no_loss_reported(&diagnostics, "DXF ByBlock import");
    let id = native.paper_layouts[0].entities[0].id();
    assert_eq!(
        native.plot_styles.entity_assignments[&id],
        ifccad::IfccadPlotStyle::ByBlock
    );
    for (extension, codec) in [("dxf", &Dxf as &dyn Format), ("dwg", &Dwg as &dyn Format)] {
        let (returned, diagnostics) = export(&native, extension, codec);
        assert_no_loss_reported(&diagnostics, "DXF ByBlock re-export");
        assert_eq!(
            cad_lines(&returned)[0].common.plotstyle_flags,
            1,
            "{extension}"
        );
    }
}

#[test]
#[ignore = "requires the original external practice table"]
fn external_plot_probe_reads_abbotsford_table() {
    let root = std::env::var_os("IFCCAD_PLOT_STYLE_PROBES").expect("probe directory");
    let root = std::path::Path::new(&root);
    let bytes = std::fs::read(root.join("1Abby.stb")).unwrap();
    assert_eq!(&bytes[19..26], b"TTBVER1");
    let table =
        opencaddocument::acadrust::plot_style::PlotStyleTable::from_bytes("1Abby.stb", &bytes)
            .expect("production codec loads the original external table");
    assert!(table.is_stb);
    assert_eq!(table.named_entries.len(), 256);
    assert!(table.named_entries.contains_key("Normal"));
    let drawing = std::fs::read(root.join("abbotsford-cover-extract.dxf")).unwrap();
    let (returned, _) = Dxf.read(&drawing).unwrap();
    assert!(!returned.header.plotstyle_mode);
    let layouts: Vec<_> = returned
        .objects
        .values()
        .filter_map(|object| match object {
            ObjectType::Layout(layout)
                if layout.plot_style_sheet.eq_ignore_ascii_case("1Abby.stb") =>
            {
                Some(layout.name.as_str())
            }
            _ => None,
        })
        .collect();
    assert!(
        !layouts.is_empty(),
        "Abbotsford refers to its original table"
    );
    let dictionary_handle = returned.get_subdictionary("ACAD_PLOTSTYLENAME").unwrap();
    let Some(ObjectType::DictionaryWithDefault(dictionary)) =
        returned.objects.get(&dictionary_handle)
    else {
        panic!("Abbotsford plot-style dictionary");
    };
    for name in dictionary.entries.iter().map(|(name, _)| name) {
        assert!(
            table
                .named_entries
                .keys()
                .any(|entry| entry.eq_ignore_ascii_case(name)),
            "Abbotsford drawing style {name:?} must exist in the original table"
        );
    }
}

#[test]
#[ignore = "generates local review files"]
fn external_plot_probe_write_drawings() {
    use opencaddocument::{
        entities::{Insert, Viewport},
        plot_style::{PlotStyleEntry, PlotStyleTable},
        tables::BlockRecord,
        LineWeight, Vector3,
    };

    let root = std::env::var_os("IFCCAD_PLOT_STYLE_PROBES").expect("probe directory");
    let root = std::path::Path::new(&root);
    let mut source = stb_drawing();
    let explicit_style = cad_lines(&source)[0].common.plotstyle_handle.unwrap();
    let dictionary = source.header.acad_plotstylename_dict_handle;
    let mut thin = PlaceHolder::new();
    thin.handle = source.allocate_handle();
    thin.owner = dictionary;
    let thin_handle = thin.handle;
    source
        .objects
        .insert(thin_handle, ObjectType::PlaceHolder(thin));
    let Some(ObjectType::DictionaryWithDefault(styles)) = source.objects.get_mut(&dictionary)
    else {
        panic!("plot-style dictionary");
    };
    styles.add_entry("Thin", thin_handle);
    source.layers.get_mut(STB_LAYER).unwrap().plotstyle_handle = thin_handle;

    let mut by_layer = Line::from_coords(10.0, 20.0, 0.0, 30.0, 20.0, 0.0);
    by_layer.common.layer = STB_LAYER.into();
    by_layer.common.color = Color::Rgb {
        r: 170,
        g: 85,
        b: 0,
    };
    by_layer.common.line_weight = LineWeight::Value(18);
    let mut explicit = Line::from_coords(10.0, 30.0, 0.0, 30.0, 30.0, 0.0);
    explicit.common.color = by_layer.common.color;
    explicit.common.plotstyle_flags = 3;
    explicit.common.plotstyle_handle = Some(explicit_style);
    let mut normal = Line::from_coords(10.0, 40.0, 0.0, 30.0, 40.0, 0.0);
    normal.common = by_layer.common.clone();
    normal.common.plotstyle_flags = 2;
    for line in [by_layer, explicit, normal] {
        source
            .add_paper_space_entity(EntityType::Line(line))
            .unwrap();
    }

    let mut block = BlockRecord::new("PLOT_PROBE");
    block.handle = source.allocate_handle();
    let block_handle = block.handle;
    source.block_records.add(block).unwrap();
    for (offset, mode, style) in [(0.0, 1, None), (10.0, 3, Some(thin_handle))] {
        let mut child = Line::from_coords(0.0, offset, 0.0, 20.0, offset, 0.0);
        child.common.owner_handle = block_handle;
        child.common.color = Color::Rgb {
            r: 170,
            g: 85,
            b: 0,
        };
        child.common.plotstyle_flags = mode;
        child.common.plotstyle_handle = style;
        source.add_entity(EntityType::Line(child)).unwrap();
    }
    let mut insert = Insert::new("PLOT_PROBE", Vector3::new(10.0, 50.0, 0.0));
    insert.common.plotstyle_flags = 3;
    insert.common.plotstyle_handle = Some(explicit_style);
    source
        .add_paper_space_entity(EntityType::Insert(insert))
        .unwrap();

    let mut ctb_table = PlotStyleTable::identity(CTB);
    for (aci, rgb, weight) in [
        (7, [0, 0, 0], 10),
        (251, [0, 255, 0], 4),
        (252, [0, 0, 255], 6),
        (253, [0, 255, 255], 8),
        (254, [255, 0, 255], 13),
        (255, [0, 0, 0], 17),
    ] {
        ctb_table.aci_entries[aci].color = Some(rgb);
        ctb_table.aci_entries[aci].lineweight = weight;
    }
    let mut stb_table = PlotStyleTable::identity(STB);
    stb_table.is_stb = true;
    for (name, color, lineweight) in [
        ("Normal", None, 0),
        ("Thin", Some([0, 0, 0]), 8),
        (STYLE, Some([0, 0, 255]), 13),
    ] {
        stb_table.named_entries.insert(
            name.into(),
            PlotStyleEntry {
                name: name.into(),
                color,
                lineweight,
                ..Default::default()
            },
        );
    }
    for table in [ctb_table, stb_table] {
        std::fs::write(root.join(&table.name), table.to_bytes().unwrap()).unwrap();
    }

    let mut ctb_source = ctb_drawing();
    // Give the True Color control the same display RGB as ACI 251.
    let control = cad_lines(&ctb_source)[6].common.handle;
    let (r, g, b) = Color::Index(251).rgb().unwrap();
    let common = ctb_source.get_entity_mut(control).unwrap().common_mut();
    common.color = Color::Rgb { r, g, b };
    common.line_weight = LineWeight::Value(18);
    let model_line = Line::from_coords(-10.0, 0.0, 0.0, 10.0, 0.0, 0.0);
    ctb_source.add_entity(EntityType::Line(model_line)).unwrap();
    let mut border = Layer::new("Viewport border");
    border.handle = ctb_source.allocate_handle();
    border.is_plottable = false;
    ctb_source.layers.add(border).unwrap();
    let mut sheet = Viewport::new();
    sheet.id = 1;
    ctb_source
        .add_paper_space_entity(EntityType::Viewport(Box::new(sheet)))
        .unwrap();
    let mut viewport = Viewport::new();
    viewport.common.layer = "Viewport border".into();
    viewport.center = Vector3::new(20.0, 90.0, 0.0);
    viewport.width = 26.0;
    viewport.height = 12.0;
    viewport.view_height = 12.0;
    viewport.id = 2;
    viewport.status.is_on = true;
    ctb_source
        .add_paper_space_entity(EntityType::Viewport(Box::new(viewport)))
        .unwrap();
    for (stem, extension, document, codec) in [
        ("ctb-paper", "dxf", ctb_source, &Dxf as &dyn Format),
        ("stb-paper", "dwg", source, &Dwg as &dyn Format),
    ] {
        let (bytes, _) = codec.write(&document).unwrap();
        let source_path = root.join(format!("{stem}.{extension}"));
        std::fs::write(&source_path, &bytes).unwrap();
        let (native, diagnostics) = import(extension, &bytes);
        assert_no_loss_reported(&diagnostics, stem);
        let encoded = encode_ifccad_document(&native).unwrap();
        std::fs::write(root.join(format!("{stem}.ifccad")), encoded.bytes()).unwrap();
        for extension in ["dxf", "dwg"] {
            let result = viewer::document_export_ifccad(
                &format!("{stem}.ifccad"),
                encoded.bytes(),
                extension,
                "AC1032",
                "{}",
            );
            assert_no_loss_reported(&result["export"]["diagnostics"], stem);
            std::fs::write(
                root.join(format!("{stem}-roundtrip.{extension}")),
                downloaded(&result),
            )
            .unwrap();
        }
    }
}
