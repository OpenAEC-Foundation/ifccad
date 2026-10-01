//! Layout settings stored with one drawing.

use super::PlotSettings;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutRect {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl LayoutRect {
    pub(crate) fn is_valid(self) -> bool {
        [self.min_x, self.min_y, self.max_x, self.max_y]
            .into_iter()
            .all(f64::is_finite)
            && self.min_x <= self.max_x
            && self.min_y <= self.max_y
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutSettings {
    pub limits: Option<LayoutRect>,
    pub limits_checking: bool,
    pub paper_space_linetype_scaling: bool,
    pub plot_settings: Option<PlotSettings>,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            limits: None,
            limits_checking: false,
            paper_space_linetype_scaling: true,
            plot_settings: None,
        }
    }
}
