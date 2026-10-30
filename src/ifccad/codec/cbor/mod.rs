//! Standalone typed CBOR mapping, independent of IFCX graph composition.
use crate::ifccad::*;
use record::RecordFields;
use token::{CborReader, CborWriter};
use value::{record, unit_enum, CborValue};
mod codes;
mod common;
mod dimension;
mod dimension_graphics;
mod dimension_style;
mod dimension_style_values;
mod document;
mod document_values;
mod filled;
mod geometry;
mod geometry_kinds;
mod hatch;
mod plot;
mod polyline_width;
mod presentation;
mod preservation;
mod record;
mod text;
mod text_columns;
mod text_format;
mod token;
mod value;
mod workspace;

pub(crate) struct EncodedCborBuffer {
    pub bytes: Vec<u8>,
    pub readback_limits: IfccadCborReadLimits,
}
pub(crate) fn documents_exactly_equal(a: &IfccadDocument, b: &IfccadDocument) -> bool {
    a.exact(b)
}
pub(crate) fn encode_bytes(
    document: &IfccadDocument,
) -> Result<EncodedCborBuffer, IfccadCborEncodeError> {
    let mut w = CborWriter::new();
    w.map(3)?;
    w.u64(0)?;
    w.str("IFCCAD-CBOR")?;
    w.u64(1)?;
    w.u64(3)?;
    w.u64(2)?;
    document.encode(&mut w)?;
    w.finish()
}
pub(crate) fn decode_bytes(
    bytes: &[u8],
    options: IfccadCborReadOptions,
) -> Result<IfccadDocument, IfccadCborReadError> {
    token::scan_one_item(bytes, options.limits)?;
    let r = CborReader::new(bytes, 0, String::new());
    let mut fields = RecordFields::new(
        &r,
        &[(0, "format"), (1, "mappingRevision"), (2, "document")],
    )?;
    let format: String = fields.take(0, None)?;
    if format != "IFCCAD-CBOR" {
        return Err(r.error("incorrect format identifier"));
    }
    let revision: u32 = fields.take(1, None)?;
    if revision != 3 {
        return Err(IfccadCborReadError::UnsupportedRevision {
            found: revision,
            supported: 3,
        });
    }
    let document = fields.take(2, None)?;
    fields.finish()?;
    Ok(document)
}

fn default_appearance() -> IfccadEntityAppearance {
    IfccadEntityAppearance {
        color: IfccadMode::ByLayer,
        opacity: IfccadMode::ByLayer,
        line_pattern: IfccadMode::ByLayer,
        line_weight: IfccadMode::ByLayer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn primitive_io_keeps_full_u64_and_binary64() {
        for value in [0.0_f64, -0.0, f64::from_bits(1), 1.0, f64::MAX] {
            let mut w = CborWriter::new();
            value.encode(&mut w).unwrap();
            let bytes = w.finish().unwrap().bytes;
            assert_eq!(bytes.len(), 9);
            assert_eq!(bytes[0], 0xfb);
            assert_eq!(
                value.to_bits(),
                f64::decode(&mut CborReader::new(&bytes, 0, "/number".into()))
                    .unwrap()
                    .to_bits()
            );
        }
        let mut w = CborWriter::new();
        u64::MAX.encode(&mut w).unwrap();
        let bytes = w.finish().unwrap().bytes;
        assert_eq!(
            u64::decode(&mut CborReader::new(&bytes, 0, "/id".into())).unwrap(),
            u64::MAX
        );
    }
    #[test]
    fn scanner_bounds_claimed_lengths() {
        assert!(token::scan_one_item(
            &[0x9b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
            Default::default()
        )
        .is_err());
        let limits = IfccadCborReadLimits {
            max_total_items: 2,
            ..Default::default()
        };
        assert!(matches!(
            token::scan_one_item(&[0xa1, 0, 1], limits),
            Err(IfccadCborReadError::ResourceLimit { .. })
        ));
    }
    #[test]
    fn record_rejects_duplicates_before_overwrite() {
        let r = CborReader::new(&[0xa2, 0, 1, 0, 2], 0, "/record".into());
        assert!(RecordFields::new(&r, &[(0, "value")]).is_err());
    }
    #[test]
    fn registry_remains_explicit() {
        let registry: serde_json::Value = serde_json::from_str(codes::MAPPING).unwrap();
        assert_eq!(registry["mappingRevision"], 3);
    }
    #[test]
    fn unregistered_dimension_definitions_are_rejected() {
        use crate::dimension::{DimensionDefinition, DimensionKind};
        assert_eq!(
            DimensionKind::decode(&mut CborReader::new(&[2], 0, "/style-kind".into())).unwrap(),
            DimensionKind::AngularTwoLines
        );
        let bytes = [0xa2, 0, 9, 1, 0xa0];
        let error =
            DimensionDefinition::decode(&mut CborReader::new(&bytes, 0, "/definition".into()))
                .unwrap_err();
        assert!(error.to_string().contains("unknown variant code"));
    }
}
