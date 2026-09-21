//! Generate small CAD inputs for a reproducible local viewer smoke test.
use ifccad_convert::cadcodec::{
    CadDocument, Circle, DwgWriter, DxfWriter, EntityType, Line, LwPolyline, Vector2,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::args_os()
        .nth(1)
        .map(std::path::PathBuf::from)
        .ok_or("provide an output directory")?;
    std::fs::create_dir_all(&root)?;
    let mut doc = CadDocument::new();
    doc.add_entity(EntityType::Line(Line::from_coords(
        0., 0., 0., 10., 10., 0.,
    )))?;
    let mut partial = Line::from_coords(5., 0., 0., 10., 5., 0.);
    partial.common.linetype_scale = 2.;
    doc.add_entity(EntityType::Line(partial))?;
    doc.add_entity(EntityType::LwPolyline(LwPolyline::from_points(vec![
        Vector2::new(0., 0.),
        Vector2::new(10., 0.),
        Vector2::new(10., 5.),
    ])))?;
    doc.add_entity(EntityType::Circle(Circle::new()))?;
    std::fs::write(
        root.join("viewer-example.dxf"),
        DxfWriter::new(&doc).write_to_vec()?,
    )?;
    std::fs::write(
        root.join("viewer-example.dwg"),
        DwgWriter::write_to_vec(&doc)?,
    )?;
    Ok(())
}
