//! Author placed geometry and a shared block directly, without an IFCX package.
use ocdraw::ocdraw::{
    BlockDefinition, BlockTransform, CoordinateFrame3, DrawingBuilder, DrawingGeometry,
    DrawingOptions, GeometricEntityDefinition, LayerDefinition, LineDefinition, Point3, RgbColor,
    Scale3, Vector3,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: write_ocdraw <new-output.ocdraw.json>")?;
    let mut drawing = DrawingBuilder::new(DrawingOptions::new("placed-geometry", "mm"))?;
    let continuous = drawing.ensure_continuous_line_pattern()?;
    let layer = drawing.add_layer(LayerDefinition::new(
        "0",
        RgbColor::new(255, 255, 255),
        continuous,
    ))?;
    let block = drawing.add_block_definition(BlockDefinition::new("Shared"))?;
    drawing
        .add_line(LineDefinition::new(layer, [0.0, 0.0, 0.0], [4.0, 3.0, 2.0]).in_scope(block))?;
    let placement = CoordinateFrame3::try_new(
        Point3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
    )?;
    drawing.add_geometric_entity(GeometricEntityDefinition::new(
        block,
        layer,
        DrawingGeometry::PlanarPolyline {
            line_pattern_generation: ocdraw::ocdraw::LinePatternGeneration::PerSegment,
            placement,
            vertices: vec![[0.0, 0.0, 0.25], [2.0, 3.0, -0.5]],
            closed: false,
        },
    ))?;
    let frame = CoordinateFrame3::try_new(
        Point3::new(10.0, 20.0, 30.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    )?;
    let transform = BlockTransform::try_new(frame, 0.4, Scale3::new(-2.0, 3.0, 1.0))?;
    drawing.add_geometric_entity(GeometricEntityDefinition::new(
        0,
        layer,
        DrawingGeometry::BlockInstance {
            definition_scope_id: block,
            transform,
        },
    ))?;
    drawing.finish()?.write_file(path)?;
    Ok(())
}
