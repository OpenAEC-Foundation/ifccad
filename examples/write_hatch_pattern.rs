use ocdraw::geometry_kernel::hatch::*;
fn fill() -> HatchFill {
    HatchFill::LinePattern(HatchLinePattern {
        name: Some("Demo families".into()),
        description: Some("Continuous and dashed/dot with signed spacing".into()),
        origin: [0.5, -0.5],
        rotation: 0.25,
        scale: 2.,
        families: vec![
            HatchLineFamily {
                angle: 0.,
                base_point: [0., 0.],
                offset: [0., 1.],
                dashes: vec![],
            },
            HatchLineFamily {
                angle: 1.2,
                base_point: [1., 2.],
                offset: [0.25, -2.5],
                dashes: vec![
                    HatchDash::Dash { length: 3. },
                    HatchDash::Gap { length: 1. },
                    HatchDash::Dot,
                ],
            },
        ],
    })
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let route = args.next().ok_or("ocdraw or ifccad required")?;
    let path = args.next().ok_or("new output path required")?;
    if route == "ocdraw" {
        use ocdraw::ocdraw::*;
        let mut d = load_ocdraw_bytes(include_bytes!("ocdraw/hello-hatch-solid.ocdraw.json"))?
            .into_document();
        d.hatch_entities[0].fill = fill();
        let bytes = encode_ocdraw_document(&d)?;
        load_ocdraw_bytes(bytes.bytes())?;
        bytes.write_file(path)?;
    } else if route == "ifccad" {
        use ocdraw::ifccad::*;
        let mut d = load_ifccad_bytes(
            include_bytes!("ifccad/hello-hatch-solid.ifcx"),
            Default::default(),
        )?
        .into_document();
        let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0]
            .as_native_mut()
            .ok_or("native Hatch required")?
            .kind
        else {
            return Err("Hatch required".into());
        };
        h.fill = fill();
        let bytes = encode_ifccad_document(&d)?;
        load_ifccad_bytes(bytes.bytes(), Default::default())?;
        bytes.write_file(path)?;
    } else {
        return Err("unknown route".into());
    }
    Ok(())
}
