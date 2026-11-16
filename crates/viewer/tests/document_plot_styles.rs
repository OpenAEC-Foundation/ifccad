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
    assert_eq!(output.plot_style_table_name.as_deref(), Some(table));
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
        let mut expected = CTB_COLORS;
        expected[1] = Color::Rgb {
            r: 254,
            g: 253,
            b: 252,
        };
        assert_eq!(cad_colors(&edited), expected, "{extension} after RGB edit");
        let layer = edited.layers.get("0").unwrap();
        assert_eq!(
            layer.color,
            Color::Rgb {
                r: 253,
                g: 252,
                b: 251
            }
        );
        assert_located_loss(
            &diagnostics,
            "ACI 251",
            cad_lines(&edited)[1].common.handle,
            extension,
        );
        assert_located_loss(&diagnostics, "ACI 252", layer.handle, extension);
    }
}

#[test]
fn stb_named_mode_survives_and_assignment_losses_are_reported() {
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
        assert_located_loss(
            &import_diagnostics,
            "plot style \"Review\"",
            line.common.handle,
            extension,
        );
        assert_located_loss(
            &import_diagnostics,
            "plot style \"Review\"",
            layer.handle,
            extension,
        );
        assert_eq!(
            import_diagnostics
                .as_array()
                .unwrap()
                .iter()
                .filter(|entry| entry["action"] == "loss")
                .count(),
            2,
            "{extension}: {import_diagnostics}",
        );
        let (returned, export_diagnostics) = export(&native, extension, codec);
        assert_cad_layout(&returned, STB);
        assert!(!returned.header.plotstyle_mode, "{extension} returned mode");
        let [line] = cad_lines(&returned)
            .try_into()
            .expect("one returned Paper line");
        assert_eq!(
            assigned_style(&returned, line),
            None,
            "{extension} returned assignment"
        );
        assert_no_loss_reported(&export_diagnostics, &format!("{extension} STB export"));
    }
}
