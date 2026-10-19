use super::{unsupported, CadHatchPreparationError};
use ifccad::geometry_kernel::hatch::HatchLoopProvenance;
use opencadcodec::entities::hatch::{BoundaryPathFlags, Hatch};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedHatchLoopFlags {
    pub provenance: Option<HatchLoopProvenance>,
    pub text_island: bool,
}

pub fn prepare_loop_flags_from_cad(
    h: &Hatch,
) -> Result<Vec<PreparedHatchLoopFlags>, CadHatchPreparationError> {
    h.paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let bits = path.flags.bits();
            if bits & !0x9f != 0 {
                return Err(unsupported(
                    "paths.flags",
                    format!("loop {index}: unqualified active loop flags {bits:#x}"),
                ));
            }
            let provenance = HatchLoopProvenance {
                from_source_objects: bits & 1 != 0,
                derived_from_pick_point: bits & 4 != 0,
                text_box: bits & 8 != 0,
                outermost_hint: bits & 16 != 0,
            };
            Ok(PreparedHatchLoopFlags {
                provenance: (!provenance.is_empty()).then_some(provenance),
                text_island: bits & 128 != 0,
            })
        })
        .collect()
}

pub fn apply_loop_flags_to_cad(
    h: &mut Hatch,
    flags: &[PreparedHatchLoopFlags],
) -> Result<(), CadHatchPreparationError> {
    if h.paths.len() != flags.len() {
        return Err(CadHatchPreparationError::LoopCountMismatch {
            expected: h.paths.len(),
            actual: flags.len(),
        });
    }
    for (index, flag) in flags.iter().enumerate() {
        if flag
            .provenance
            .as_ref()
            .is_some_and(HatchLoopProvenance::is_empty)
        {
            return Err(unsupported(
                "paths.provenance",
                format!("loop {index}: empty provenance must be omitted"),
            ));
        }
    }
    for (path, flag) in h.paths.iter_mut().zip(flags) {
        let mut bits = path.flags.bits() & BoundaryPathFlags::POLYLINE.bits();
        if let Some(p) = &flag.provenance {
            bits |= u32::from(p.from_source_objects)
                | (u32::from(p.derived_from_pick_point) << 2)
                | (u32::from(p.text_box) << 3)
                | (u32::from(p.outermost_hint) << 4);
        }
        bits |= u32::from(flag.text_island) << 7;
        path.flags = BoundaryPathFlags::from_bits(bits);
    }
    Ok(())
}
