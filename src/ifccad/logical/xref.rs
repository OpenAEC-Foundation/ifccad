use super::*;

/// Whether an external drawing propagates when its owner is referenced.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum IfccadXrefReferenceType {
    #[default]
    Attach,
    Overlay,
}

/// Placement maps source coordinates relative to its base point, scaled once.
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadXref {
    pub source_id: IfccadExternalSourceId,
    pub transform: IfccadBlockTransform,
    pub reference_type: IfccadXrefReferenceType,
}

impl IfccadXref {
    pub(super) fn validate_at(
        &self,
        document: &IfccadDocument,
        path: &str,
    ) -> Result<(), IfccadReport> {
        let frame = self.transform.placement.coordinate_frame()?;
        crate::geometry_kernel::BlockTransform::try_new(
            frame,
            self.transform.rotation,
            crate::geometry_kernel::Scale3::new(
                self.transform.scale[0],
                self.transform.scale[1],
                self.transform.scale[2],
            ),
        )
        .map_err(|e| {
            crate::ifccad::diagnostics::failure(
                "IFCCAD-XREF-001",
                path,
                format!("invalid XREF transform: {e}"),
            )
        })?;
        if !document
            .external_sources
            .iter()
            .any(|s| s.id == self.source_id)
        {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-XREF-001",
                path,
                "XREF requires a drawing-local source",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry_kernel::{BlockTransform, Point3, PreparedBlockTransform, Scale3};

    #[test]
    fn source_base_is_anchor_and_oblique_signed_axes_follow_block_semantics() {
        let transform = IfccadBlockTransform {
            placement: IfccadPlacement {
                origin: [10., 20., 30.],
                x_axis: [1., 0., 0.],
                y_axis: [0., 0., -1.],
            },
            rotation: 0.5,
            scale: [-2., 3., 4.],
        };
        let t = BlockTransform::try_new(
            transform.placement.coordinate_frame().unwrap(),
            transform.rotation,
            Scale3::new(-2., 3., 4.),
        )
        .unwrap();
        let p = PreparedBlockTransform::new(t, Point3::new(1., 2., 3.)).unwrap();
        let anchor = p.apply(Point3::new(1., 2., 3.)).unwrap();
        for (axis, expected) in anchor.iter().zip([10., 20., 30.]) {
            assert_eq!(axis.lower, expected);
            assert_eq!(axis.upper, expected);
        }
        let (sin, cos) = 0.5_f64.sin_cos();
        for (point, expected) in [
            ([2., 2., 3.], [10. - 2. * cos, 20., 30. + 2. * sin]),
            ([1., 3., 3.], [10. - 3. * sin, 20., 30. - 3. * cos]),
            ([1., 2., 4.], [10., 24., 30.]),
        ] {
            let result = p.apply(Point3::new(point[0], point[1], point[2])).unwrap();
            // Independent rounded witnesses check orientation/order, not a new numeric acceptance tolerance.
            for (axis, value) in result.iter().zip(expected) {
                assert!((axis.lower - value).abs() < 1e-12);
                assert!((axis.upper - value).abs() < 1e-12);
            }
        }
    }
}
