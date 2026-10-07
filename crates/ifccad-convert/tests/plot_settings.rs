mod common;
use ifccad_convert::{
    cad_document_to_ifccad_document,
    opencadcodec::{objects::ObjectType, CadDocument},
};
#[test]
fn physical_medium_and_plot_mapping_are_independent() {
    let mut cad = CadDocument::new();
    for o in cad.objects.values_mut() {
        if let ObjectType::Layout(l) = o {
            if l.name == "Layout1" {
                l.paper_width = 279.4;
                l.paper_height = 215.9;
                l.plot_paper_units = 0;
                l.plot_type = 1;
                l.plot_scale_numerator = 2.;
                l.plot_scale_denominator = 1.;
                l.plot_flags.use_standard_scale = false;
                l.plot_origin_x = 127.;
            }
        }
    }
    let out =
        cad_document_to_ifccad_document(&cad, common::metadata(), Default::default()).unwrap();
    let paper = &out.document().paper_layouts[0];
    assert_eq!(paper.settings.media.as_ref().unwrap().width, 279.4);
    assert_eq!(paper.settings.media.as_ref().unwrap().unit.as_str(), "mm");
    assert_eq!(
        paper.settings.plot_settings.as_ref().unwrap().plot_unit,
        ocdraw::plot_kernel::PlotUnit::Inch
    );
}
