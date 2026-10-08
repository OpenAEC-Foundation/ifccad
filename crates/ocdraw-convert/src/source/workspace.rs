//! Selection qualification on the pinned CAD public surface.
use ocdraw::ocdraw::{DrawingModelWindow, DrawingUcsSelection};
pub(crate) fn active_model_window(
    windows: &[DrawingModelWindow],
    current: Option<DrawingUcsSelection>,
) -> Option<u32> {
    // One *ACTIVE record is unambiguous. Multiple-record ordering has not been
    // qualified across both transports; preserve values without assigning a choice.
    let [window] = windows else { return None };
    if window.use_stored_ucs && current.is_some_and(|ucs| ucs != window.stored_ucs) {
        None
    } else {
        Some(window.id)
    }
}
