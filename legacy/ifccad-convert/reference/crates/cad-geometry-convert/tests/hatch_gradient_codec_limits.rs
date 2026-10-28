//! Narrow characterization of source information unavailable to adapters.
use opencadcodec::io::dwg::{
    dwg_stream_readers::bit_reader::DwgBitReader, dwg_stream_writers::bit_writer::DwgBitWriter,
    dwg_version::DwgVersion,
};
use opencadcodec::{Color, DxfReader, DxfVersion};
use std::io::Cursor;
fn read(text: &str) -> opencadcodec::entities::Hatch {
    let d = DxfReader::from_reader(Cursor::new(text.as_bytes().to_vec()))
        .unwrap()
        .read()
        .unwrap();
    let h = d
        .entities()
        .find_map(|e| {
            if let opencadcodec::EntityType::Hatch(h) = e {
                Some(h.clone())
            } else {
                None
            }
        })
        .unwrap();
    *h
}
#[test]
fn pinned_dxf_reader_ignores_declared_count_but_true_color_precedes_aci() {
    // Git may check out the literal fixture with CRLF on Windows. Normalize
    // line endings so each mutation really changes the intended wire tags.
    let text = include_str!("fixtures/hatch-gradient-reference.dxf").replace("\r\n", "\n");
    for tags in ["453\n2\n", "63\n1\n421\n14438430\n", "460\n0.75\n"] {
        assert_eq!(text.matches(tags).count(), 1, "missing or ambiguous tags");
    }
    let mismatch = read(&text.replace("453\n2\n", "453\n9\n"));
    assert_eq!(mismatch.gradient_color.colors.len(), 2);
    let reversed = read(&text.replace("63\n1\n421\n14438430\n", "421\n14438430\n63\n1\n"));
    assert_eq!(
        reversed.gradient_color.colors[0].color,
        Color::Rgb {
            r: 220,
            g: 80,
            b: 30
        }
    );
    let missing = read(&text.replace("460\n0.75\n", ""));
    assert_eq!(missing.gradient_color.angle, 0.0);
}
#[test]
fn pinned_cmc_reader_consumes_names_but_exposes_only_concrete_color() {
    // Independent CMC packet: legacy index, C2 RGB, name/book flags, strings,
    // sentinel. This is not a complete DWG-file qualification.
    let mut w = DwgBitWriter::new(DwgVersion::AC18, DxfVersion::AC1018);
    w.write_bit_short(0);
    w.write_bit_long(0xC2123456_u32 as i32);
    w.write_byte(3);
    w.write_variable_text("Ink");
    w.write_variable_text("Book");
    w.write_bit_long(12345);
    let mut r = DwgBitReader::new(w.into_bytes(), DwgVersion::AC18, DxfVersion::AC1018);
    assert_eq!(
        r.read_cm_color(),
        Color::Rgb {
            r: 0x12,
            g: 0x34,
            b: 0x56
        }
    );
    assert_eq!(r.read_bit_long(), 12345);
}
