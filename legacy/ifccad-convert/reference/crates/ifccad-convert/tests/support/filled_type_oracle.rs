//! Bounded wire oracle, independent of CadDocument entity construction.
use opencadcodec::io::dwg::dwg_stream_readers::{
    handle_reader::read_handles, object_reader::DwgObjectReader,
};
use opencadcodec::{types::DxfVersion, DwgReader};
use std::io::Cursor;
pub fn ascii_tags(bytes: &[u8]) -> Vec<(i16, String)> {
    let s = std::str::from_utf8(bytes).unwrap();
    let lines: Vec<_> = s.lines().collect();
    assert_eq!(lines.len() % 2, 0);
    lines
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| (c[0].trim().parse().unwrap(), c[1].trim().to_owned()))
        .collect()
}
pub fn binary_tags(bytes: &[u8]) -> Vec<(i16, String)> {
    assert!(bytes.starts_with(b"AutoCAD Binary DXF\r\n\x1a\0"));
    let mut i = 22;
    let mut tags = Vec::new();
    while i < bytes.len() {
        let code = i16::from_le_bytes(bytes[i..i + 2].try_into().unwrap());
        i += 2;
        let v = match code {
            0..=9
            | 100
            | 102
            | 105
            | 300..=309
            | 320..=369
            | 390..=399
            | 410..=419
            | 430..=439
            | 470..=481
            | 999..=1003
            | 1005 => {
                let n = bytes[i..].iter().position(|b| *b == 0).unwrap();
                let v = String::from_utf8_lossy(&bytes[i..i + n]).into_owned();
                i += n + 1;
                v
            }
            10..=59 | 110..=149 | 210..=239 | 460..=469 | 1010..=1059 => {
                let v = f64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
                i += 8;
                v.to_string()
            }
            60..=79 | 170..=179 | 270..=289 | 370..=389 | 400..=409 | 1060..=1070 => {
                let v = i16::from_le_bytes(bytes[i..i + 2].try_into().unwrap());
                i += 2;
                v.to_string()
            }
            90..=99 | 420..=429 | 440..=459 | 1071 => {
                let v = i32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
                i += 4;
                v.to_string()
            }
            160..=169 => {
                let v = i64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
                i += 8;
                v.to_string()
            }
            290..=299 => {
                let v = bytes[i];
                i += 1;
                v.to_string()
            }
            310..=319 | 1004 => {
                let n = bytes[i] as usize;
                i += 1 + n;
                String::new()
            }
            _ => panic!("unqualified binary tag {code}"),
        };
        tags.push((code, v));
    }
    tags
}
pub fn ascii_entity_types(bytes: &[u8]) -> Vec<String> {
    ascii_tags(bytes)
        .into_iter()
        .filter_map(|(c, v)| (c == 0).then_some(v))
        .collect()
}
pub fn binary_entity_types(bytes: &[u8]) -> Vec<String> {
    binary_tags(bytes)
        .into_iter()
        .filter_map(|(c, v)| (c == 0).then_some(v))
        .collect()
}
pub fn dwg_object_types(bytes: &[u8]) -> Vec<i16> {
    let mut reader = DwgReader::from_stream(Cursor::new(bytes));
    let header = reader.read_file_header().unwrap();
    let mut handles =
        read_handles(&reader.get_section_buffer("AcDb:Handles", &header).unwrap()).unwrap();
    for offset in handles.values_mut() {
        *offset -= header.objects_base_offset;
    }
    let offsets: Vec<_> = handles.values().copied().collect();
    let objects = DwgObjectReader::new(
        reader
            .get_section_buffer("AcDb:AcDbObjects", &header)
            .unwrap(),
        DxfVersion::AC1032,
        handles,
    )
    .unwrap();
    offsets
        .iter()
        .map(|&offset| objects.type_code_at(offset as usize).unwrap())
        .collect()
}
