pub fn tags(bytes: &[u8]) -> Vec<(i32, String)> {
    if !bytes.starts_with(b"AutoCAD Binary DXF") {
        let lines: Vec<_> = std::str::from_utf8(bytes).unwrap().lines().collect();
        return lines
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| (p[0].trim().parse().unwrap(), p[1].trim().to_owned()))
            .collect();
    }
    let mut i = 22;
    let mut pairs = Vec::new();
    while i < bytes.len() {
        let code = u16::from_le_bytes(bytes[i..i + 2].try_into().unwrap()) as i32;
        i += 2;
        let size = match code {
            10..=59 | 110..=149 | 210..=239 | 460..=469 | 1010..=1059 => 8,
            60..=79 | 170..=179 | 270..=279 | 370..=389 | 400..=409 | 1060..=1070 => 2,
            90..=99 | 420..=429 | 440..=459 | 1071 => 4,
            160..=169 => 8,
            280..=299 => 1,
            310..=319 | 1004 => {
                let n = bytes[i] as usize;
                i += 1;
                i += n;
                pairs.push((code, String::new()));
                continue;
            }
            _ => 0,
        };
        let value = if size == 0 {
            let n = bytes[i..].iter().position(|b| *b == 0).unwrap();
            let s = String::from_utf8_lossy(&bytes[i..i + n]).into_owned();
            i += n + 1;
            s
        } else {
            let data = &bytes[i..i + size];
            i += size;
            if matches!(code,10..=59|110..=149|210..=239|460..=469|1010..=1059) {
                f64::from_le_bytes(data.try_into().unwrap()).to_string()
            } else {
                String::new()
            }
        };
        let eof = code == 0 && value == "EOF";
        pairs.push((code, value));
        if eof {
            break;
        }
    }
    pairs
}

pub fn records(tags: &[(i32, String)], kind: &str) -> Vec<Vec<(i32, String)>> {
    let mut result = Vec::new();
    let mut current = None;
    for pair in tags {
        if pair.0 == 0 {
            if let Some(r) = current.take() {
                result.push(r)
            };
            if pair.1 == kind {
                current = Some(Vec::new())
            }
        } else if let Some(r) = &mut current {
            r.push(pair.clone())
        }
    }
    if let Some(r) = current {
        result.push(r)
    }
    result
}

pub fn actual_header_version(bytes: &[u8], binary: bool) -> String {
    assert_eq!(
        bytes.starts_with(b"AutoCAD Binary DXF\r\n\x1a\0"),
        binary,
        "actual DXF representation differs from selected format"
    );
    let t = tags(bytes);
    let i = t
        .iter()
        .position(|p| p.0 == 9 && p.1 == "$ACADVER")
        .unwrap();
    assert_eq!(t[i + 1].0, 1);
    t[i + 1].1.clone()
}
pub fn classic_width_tags(bytes: &[u8], binary: bool) -> ([f64; 2], Vec<[Option<f64>; 2]>) {
    assert_eq!(
        bytes.starts_with(b"AutoCAD Binary DXF\r\n\x1a\0"),
        binary,
        "actual DXF representation differs from selected format"
    );
    let t = tags(bytes);
    let p = records(&t, "POLYLINE");
    assert_eq!(p.len(), 1, "ambiguous classic selection");
    let val = |r: &Vec<(i32, String)>, c| {
        let vs: Vec<_> = r.iter().filter(|x| x.0 == c).collect();
        assert!(vs.len() <= 1);
        vs.first().map(|x| x.1.parse::<f64>().unwrap())
    };
    let id = p[0].iter().find(|x| x.0 == 5).unwrap().1.clone();
    let v = records(&t, "VERTEX");
    let mut handles = std::collections::BTreeSet::new();
    let leaves = v
        .iter()
        .map(|r| {
            assert_eq!(r.iter().find(|x| x.0 == 330).unwrap().1, id);
            assert!(handles.insert(r.iter().find(|x| x.0 == 5).unwrap().1.clone()));
            [val(r, 40), val(r, 41)]
        })
        .collect();
    (
        [val(&p[0], 40).unwrap_or(0.), val(&p[0], 41).unwrap_or(0.)],
        leaves,
    )
}
// ODA Open Design Specification 5.4.1: POLYLINE(2D) and VERTEX(2D).
// Generic record/framing only; no codec entity width decoder or builder.
pub fn dwg_classic_widths(
    bytes: &[u8],
    version: opencadcodec::DxfVersion,
) -> ([f64; 2], Vec<[f64; 2]>) {
    use opencadcodec::io::dwg::dwg_stream_readers::bit_reader::DwgBitReader;
    use opencadcodec::io::dwg::dwg_stream_readers::{
        handle_reader::read_handles, object_reader::DwgObjectReader,
    };
    fn bd(r: &mut DwgBitReader) -> f64 {
        match r.read_2bits() {
            0 => r.read_raw_double(),
            1 => 1.,
            2 => 0.,
            _ => panic!("invalid BD"),
        }
    }
    let mut r = opencadcodec::DwgReader::from_stream(std::io::Cursor::new(bytes));
    let h = r.read_file_header().unwrap();
    let mut handles = read_handles(&r.get_section_buffer("AcDb:Handles", &h).unwrap()).unwrap();
    for o in handles.values_mut() {
        *o -= h.objects_base_offset;
    }
    let offsets: Vec<_> = handles.values().copied().collect();
    let objects = DwgObjectReader::new(
        r.get_section_buffer("AcDb:AcDbObjects", &h).unwrap(),
        version,
        handles,
    )
    .unwrap();
    let mut headers = vec![];
    let mut vertices = vec![];
    let mut parent = None;
    let mut children = std::collections::BTreeMap::new();
    for o in offsets {
        let (ty, mut rr) = objects.read_record_at(o as usize).unwrap();
        if ty != 15 && ty != 10 {
            continue;
        }
        let common = objects.read_common_entity_data(&mut rr, ty);
        let b = rr.main_mut();
        if ty == 15 {
            assert!(parent.replace(common.common.handle).is_none());
            b.read_bit_short();
            b.read_bit_short();
            headers.push([bd(b), bd(b)]);
        } else {
            assert!(children
                .insert(common.common.handle, common.owner_handle)
                .is_none());
            b.read_byte();
            let x = bd(b);
            bd(b);
            bd(b);
            let sw = bd(b);
            let pair = if sw < 0. { [-sw, -sw] } else { [sw, bd(b)] };
            vertices.push((x, pair));
        }
    }
    assert_eq!(headers.len(), 1, "ambiguous classic selection");
    assert!(
        children.values().all(|owner| Some(*owner) == parent),
        "DWG child owner link mismatch"
    );
    vertices.sort_by(|a, b| a.0.total_cmp(&b.0));
    (headers[0], vertices.into_iter().map(|v| v.1).collect())
}

// LWPOLYLINE(77) raw optional-array grammar; primitive DD skips only point data.
pub fn dwg_lw_widths(bytes: &[u8], version: opencadcodec::DxfVersion) -> (f64, Vec<[f64; 2]>) {
    use opencadcodec::io::dwg::dwg_stream_readers::{
        bit_reader::DwgBitReader, handle_reader::read_handles, object_reader::DwgObjectReader,
    };
    fn bd(r: &mut DwgBitReader) -> f64 {
        match r.read_2bits() {
            0 => r.read_raw_double(),
            1 => 1.,
            2 => 0.,
            _ => panic!("invalid BD"),
        }
    }
    let mut reader = opencadcodec::DwgReader::from_stream(std::io::Cursor::new(bytes));
    let header = reader.read_file_header().unwrap();
    let mut handles =
        read_handles(&reader.get_section_buffer("AcDb:Handles", &header).unwrap()).unwrap();
    for o in handles.values_mut() {
        *o -= header.objects_base_offset;
    }
    let offsets: Vec<_> = handles.values().copied().collect();
    let objects = DwgObjectReader::new(
        reader
            .get_section_buffer("AcDb:AcDbObjects", &header)
            .unwrap(),
        version,
        handles,
    )
    .unwrap();
    let mut rows = vec![];
    for o in offsets {
        if objects.type_code_at(o as usize).unwrap() != 77 {
            continue;
        }
        let (ty, mut r) = objects.read_record_at(o as usize).unwrap();
        objects.read_common_entity_data(&mut r, ty);
        let b = r.main_mut();
        let flags = b.read_bit_short();
        if flags & 0x24 == 0 {
            continue;
        }
        assert_ne!(flags & 0x24, 0x24, "competing constant/vertex controls");
        let constant = if flags & 4 != 0 { bd(b) } else { 0. };
        if flags & 8 != 0 {
            bd(b);
        }
        if flags & 2 != 0 {
            bd(b);
        }
        if flags & 1 != 0 {
            for _ in 0..3 {
                bd(b);
            }
        }
        let n = b.read_bit_long();
        assert!((2..=100).contains(&n));
        let bulges = if flags & 0x10 != 0 {
            b.read_bit_long()
        } else {
            0
        };
        let ids = if flags & 0x400 != 0 {
            b.read_bit_long()
        } else {
            0
        };
        let widths = if flags & 0x20 != 0 {
            b.read_bit_long()
        } else {
            0
        };
        let mut point = [b.read_raw_double(), b.read_raw_double()];
        for _ in 1..n {
            point = [
                b.read_bit_double_with_default(point[0]),
                b.read_bit_double_with_default(point[1]),
            ];
        }
        for _ in 0..bulges {
            bd(b);
        }
        for _ in 0..ids {
            b.read_bit_long();
        }
        let pairs = (0..widths).map(|_| [bd(b), bd(b)]).collect();
        rows.push((constant, pairs));
    }
    assert_eq!(rows.len(), 1, "ambiguous nonzero LW selection");
    rows.pop().unwrap()
}
