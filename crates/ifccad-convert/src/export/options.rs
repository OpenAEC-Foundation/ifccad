#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ExportLossPolicy {
    #[default]
    Allow,
    Reject,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExportOptions {
    pub loss_policy: ExportLossPolicy,
}
