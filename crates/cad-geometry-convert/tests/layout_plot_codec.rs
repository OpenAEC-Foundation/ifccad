use opencadcodec::{objects::ObjectType, CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter};
use std::io::Cursor;

fn layout(doc: &CadDocument, name: &str) -> opencadcodec::objects::Layout {
    doc.objects
        .values()
        .find_map(|o| match o {
            ObjectType::Layout(l) if l.name == name => Some(l.clone()),
            _ => None,
        })
        .unwrap()
}

#[test]
fn literal_millimetre_fields_do_not_follow_inch_plot_selector() {
    let doc = DxfReader::from_reader(Cursor::new(include_bytes!(
        "fixtures/layout-plot-reference.dxf"
    )))
    .unwrap()
    .read()
    .unwrap();
    let l = layout(&doc, "InchOutput");
    assert_eq!(l.paper_width, 279.4);
    assert_eq!(l.plot_paper_units, 0);
    assert_eq!(l.plot_margin_left, 5.0);
    assert_eq!(l.plot_origin_x, 127.0);
    assert!(!l.plot_flags.use_standard_scale);
    assert_eq!(l.plot_scale_type, 0);
    assert_eq!(l.plot_scale_numerator, 2.0);
    assert_eq!(l.plot_scale_denominator, 1.0);
    assert_eq!(l.flags & 3, 2);
}

#[test]
fn layout_flags_and_custom_scale_survive_dxf_and_dwg() {
    let mut doc = CadDocument::new();
    doc.add_layout("Second").unwrap();
    for o in doc.objects.values_mut() {
        if let ObjectType::Layout(l) = o {
            if l.name != "Model" {
                l.paper_width = 420.0;
                l.paper_height = 297.0;
                l.plot_paper_units = 1;
                l.plot_scale_type = 0;
                l.plot_flags.use_standard_scale = false;
                l.plot_scale_numerator = 2.0;
                l.plot_scale_denominator = 1.0;
                l.plot_scale_factor = 2.0;
                l.flags = if l.name == "Second" { 3 } else { 0 };
            }
        }
    }
    for dwg in [false, true] {
        let back = if dwg {
            DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(&doc).unwrap()))
                .read()
                .unwrap()
        } else {
            DxfReader::from_reader(Cursor::new(DxfWriter::new(&doc).write_to_vec().unwrap()))
                .unwrap()
                .read()
                .unwrap()
        };
        for name in ["Layout1", "Second"] {
            let before = layout(&doc, name);
            let after = layout(&back, name);
            assert_eq!(after.flags & 3, before.flags & 3, "{dwg} {name}");
            assert_eq!(after.plot_scale_type, 0);
            assert!(!after.plot_flags.use_standard_scale);
            assert_eq!(
                (after.plot_scale_numerator, after.plot_scale_denominator),
                (2.0, 1.0)
            );
            assert_eq!(after.paper_width, 420.0);
        }
    }
}

#[test]
fn medium_only_standard_and_pixel_fields_remain_explicit() {
    let mut doc = CadDocument::new();
    for o in doc.objects.values_mut() {
        if let ObjectType::Layout(l) = o {
            if l.name == "Layout1" {
                l.paper_width = 420.;
                l.paper_height = 297.;
            }
        }
    }
    for (standard, pixel) in [(false, false), (true, false), (false, true)] {
        let mut source = doc.clone();
        for o in source.objects.values_mut() {
            if let ObjectType::Layout(l) = o {
                if l.name == "Layout1" {
                    if standard {
                        l.plot_flags.use_standard_scale = true;
                        l.plot_scale_type = 16;
                    }
                    if pixel {
                        l.plot_paper_units = 2;
                    }
                }
            }
        }
        for dwg in [false, true] {
            let back = if dwg {
                DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(&source).unwrap()))
                    .read()
                    .unwrap()
            } else {
                DxfReader::from_reader(Cursor::new(DxfWriter::new(&source).write_to_vec().unwrap()))
                    .unwrap()
                    .read()
                    .unwrap()
            };
            let before = layout(&source, "Layout1");
            let after = layout(&back, "Layout1");
            assert_eq!(after.paper_width, before.paper_width);
            assert_eq!(after.plot_paper_units, before.plot_paper_units);
            assert_eq!(after.plot_scale_type, before.plot_scale_type);
            assert_eq!(
                after.plot_flags.use_standard_scale,
                before.plot_flags.use_standard_scale
            );
        }
    }
}
