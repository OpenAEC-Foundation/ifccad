use std::num::NonZeroU64;

/// Constraint on the exact signed scale factors of every instance of a definition.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlockScaling {
    #[default]
    Any,
    Uniform,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2 {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl Point2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn x(self) -> f64 {
        self.x
    }

    pub fn y(self) -> f64 {
        self.y
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds2d {
    pub(crate) min: Point2,
    pub(crate) max: Point2,
}

impl Bounds2d {
    pub fn min(self) -> Point2 {
        self.min
    }

    pub fn max(self) -> Point2 {
        self.max
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LayerId(u32);

impl LayerId {
    pub(crate) fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for LayerId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AppearanceId(u32);

impl AppearanceId {
    pub(crate) fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for AppearanceId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EntityId(NonZeroU64);

impl EntityId {
    /// Constructs a nonzero entity identity. Resource validation also checks uniqueness.
    ///
    /// ```
    /// use ifccad::ifcdr::EntityId;
    /// let existing_id = EntityId::new(42).unwrap();
    /// assert_eq!(existing_id.get(), 42);
    /// assert!(EntityId::new(0).is_none());
    /// ```
    pub fn new(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }

    pub fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ScopeId(u32);

impl ScopeId {
    pub(crate) fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

/// Coordinate length unit owned and declared by an IFCDR resource.
///
/// It is resource metadata rather than a package- or drawing-wide setting;
/// future packages may therefore contain resources with different units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IfcdrLengthUnit {
    Unitless,
    Millimetre,
    Centimetre,
    Metre,
    Kilometre,
    Inch,
    Foot,
    Mile,
    Microinch,
    Mil,
    Yard,
    Angstrom,
    Nanometre,
    Micrometre,
    Decimetre,
    Decametre,
    Hectometre,
    Gigametre,
    AstronomicalUnit,
    LightYear,
    Parsec,
    UsSurveyFoot,
    UsSurveyInch,
    UsSurveyYard,
    UsSurveyMile,
}

#[cfg(test)]
mod length_unit_tests {
    #[test]
    fn all_contract_unit_tokens_parse_and_encode_without_aliases() {
        let registry: serde_json::Value =
            serde_json::from_str(include_str!("../../schemas/ifcdr/registry-0.9.0.json")).unwrap();
        let tokens = registry["types"]["unit"]["values"].as_array().unwrap();
        assert_eq!(tokens.len(), 25);
        for token in tokens {
            let token = token.as_str().unwrap();
            let unit = crate::ifcdr::logical::length_unit(token)
                .unwrap_or_else(|| panic!("missing {token}"));
            assert_eq!(crate::ifcdr::codec::json::unit_name(unit), token);
        }
        for alias in ["GM", "metre", "usSurveyfoot", "M", "µm", ""] {
            assert!(crate::ifcdr::logical::length_unit(alias).is_none());
        }
    }
}
