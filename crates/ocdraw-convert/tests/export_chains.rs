mod support;

use support::documents::{
    assert_export_chain, assert_minimal_ifccad_chain, loss_heavy_document,
    supported_model_space_document,
};

#[test]
fn supported_document_crosses_dxf_ifccad_dxf_without_export_loss() {
    assert_export_chain(supported_model_space_document(), false);
}

#[test]
fn loss_heavy_document_reports_the_complete_allow_and_reject_lists() {
    assert_export_chain(loss_heavy_document(), true);
}

#[test]
fn bundled_ifccad_crosses_caddocument_dxf_and_back_to_ifccad() {
    assert_minimal_ifccad_chain();
}
