use serde_json::Value;

#[derive(Clone, Copy)]
pub(super) struct UInt64Column<'a>(&'a [Value]);
impl<'a> UInt64Column<'a> {
    pub(super) fn new(values: &'a [Value]) -> Self {
        Self(values)
    }
    pub(super) fn get(self, index: usize) -> Option<u64> {
        self.0.get(index)?.as_u64()
    }
}

#[derive(Clone, Copy)]
pub(super) struct UInt32Column<'a>(&'a [Value]);
impl<'a> UInt32Column<'a> {
    pub(super) fn new(values: &'a [Value]) -> Self {
        Self(values)
    }
    pub(super) fn get(self, index: usize) -> Option<u32> {
        u32::try_from(self.0.get(index)?.as_u64()?).ok()
    }
}

#[derive(Clone, Copy)]
pub(super) struct Float64Column<'a>(&'a [Value]);
impl<'a> Float64Column<'a> {
    pub(super) fn new(values: &'a [Value]) -> Self {
        Self(values)
    }
    pub(super) fn get(self, index: usize) -> Option<f64> {
        self.0
            .get(index)?
            .as_f64()
            .filter(|value| value.is_finite())
    }
}

#[derive(Clone, Copy)]
pub(super) struct BooleanColumn<'a> {
    values: Option<&'a [Value]>,
    omission_default: Option<bool>,
}
impl<'a> BooleanColumn<'a> {
    pub(super) fn new(values: Option<&'a [Value]>, omission_default: Option<bool>) -> Self {
        Self {
            values,
            omission_default,
        }
    }
    pub(super) fn get(self, index: usize) -> Option<bool> {
        match self.values {
            Some(values) => values.get(index)?.as_bool(),
            None => self.omission_default,
        }
    }
}
