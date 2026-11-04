//! Produce one bounded native source gallery for manual CAD roundtrip review.
#[path = "support/font_symbol_gallery.rs"]
mod gallery;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let destination = std::path::PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("usage: font_symbol_gallery NEW_DIRECTORY")?,
    );
    if destination.exists() {
        return Err("review destination must be new".into());
    }
    let bytes = gallery::build()?;
    let loaded = ifccad::load_ifccad_bytes(&bytes, Default::default())?;
    let cbor = ifccad::encode_ifccad_cbor_document(loaded.document())?;
    std::fs::create_dir_all(&destination)?;
    std::fs::write(destination.join("font-symbol-gallery.ifcx"), bytes)?;
    std::fs::write(
        destination.join("font-symbol-gallery.ifccad.cbor"),
        cbor.bytes(),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn gallery_has_distinct_sources_and_saved_annotation_contexts() {
        let bytes = super::gallery::build().unwrap();
        let loaded = ifccad::load_ifccad_bytes(&bytes, Default::default()).unwrap();
        let d = loaded.document();
        assert!(d.annotation_scales.len() >= 2);
        assert!(d
            .symbol_sources
            .iter()
            .any(|s| s.source.effective_cad_file_name().starts_with("..\\")));
        assert!(d
            .symbol_sources
            .iter()
            .any(|s| s.source.effective_cad_file_name().starts_with("\\\\")));
        let bare: Vec<_> = d
            .symbol_sources
            .iter()
            .filter(|s| s.source.effective_cad_file_name() == "ltypeshp.shx")
            .collect();
        assert_eq!(bare.len(), 2);
        assert_ne!(bare[0].id, bare[1].id);
        assert!(d.text_styles.iter().any(
            |s| s.properties.font.bold == Some(true) && s.properties.font.italic == Some(true)
        ));
    }
}
