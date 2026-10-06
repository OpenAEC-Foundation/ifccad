use base64::{engine::general_purpose::STANDARD, Engine};
use opencadcodec::entities::EntityCommon;
use opencadcodec::xdata::{ExtendedData, ExtendedDataRecord, XDataValue};
use opencadcodec::{Color, Handle, LineWeight, Transparency, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub(crate) struct SnapshotFloat(pub f64);
impl Serialize for SnapshotFloat {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("{:016x}", self.0.to_bits()))
    }
}
impl<'de> Deserialize<'de> for SnapshotFloat {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        if text.len() != 16
            || !text
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(serde::de::Error::custom(
                "binary64 requires 16 lowercase hexadecimal digits",
            ));
        }
        u64::from_str_radix(&text, 16)
            .map(|v| Self(f64::from_bits(v)))
            .map_err(serde::de::Error::custom)
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct SnapshotHandle(pub Handle);
impl Serialize for SnapshotHandle {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("{:x}", self.0))
    }
}
impl<'de> Deserialize<'de> for SnapshotHandle {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        let value = u64::from_str_radix(&text, 16).map_err(serde::de::Error::custom)?;
        if text != format!("{value:x}") {
            return Err(serde::de::Error::custom("noncanonical source handle"));
        }
        Ok(Self(Handle::new(value)))
    }
}
#[derive(Clone, Debug)]
pub(crate) struct SnapshotBytes(pub Vec<u8>);
impl Serialize for SnapshotBytes {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&STANDARD.encode(&self.0))
    }
}
impl<'de> Deserialize<'de> for SnapshotBytes {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        let bound = text
            .len()
            .checked_div(4)
            .and_then(|n| n.checked_mul(3))
            .ok_or_else(|| serde::de::Error::custom("byte size overflow"))?;
        let bytes = STANDARD.decode(&text).map_err(serde::de::Error::custom)?;
        if bytes.len() > bound || STANDARD.encode(&bytes) != text {
            return Err(serde::de::Error::custom("noncanonical bytes"));
        }
        Ok(Self(bytes))
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub(crate) struct SnapshotVector(pub [SnapshotFloat; 3]);
impl SnapshotVector {
    pub fn capture(v: Vector3) -> Self {
        Self([SnapshotFloat(v.x), SnapshotFloat(v.y), SnapshotFloat(v.z)])
    }
    pub fn restore(self) -> Vector3 {
        Vector3::new(self.0[0].0, self.0[1].0, self.0[2].0)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum SnapshotColor {
    ByLayer,
    None,
    ByBlock,
    Index(u8),
    Rgb { r: u8, g: u8, b: u8 },
}
impl SnapshotColor {
    fn capture(v: Color) -> Self {
        match v {
            Color::ByLayer => Self::ByLayer,
            Color::None => Self::None,
            Color::ByBlock => Self::ByBlock,
            Color::Index(v) => Self::Index(v),
            Color::Rgb { r, g, b } => Self::Rgb { r, g, b },
        }
    }
    fn restore(&self) -> Color {
        match *self {
            Self::ByLayer => Color::ByLayer,
            Self::None => Color::None,
            Self::ByBlock => Color::ByBlock,
            Self::Index(v) => Color::Index(v),
            Self::Rgb { r, g, b } => Color::Rgb { r, g, b },
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum SnapshotWeight {
    ByLayer,
    ByBlock,
    Default,
    Value(i16),
}
impl SnapshotWeight {
    fn capture(v: LineWeight) -> Self {
        match v {
            LineWeight::ByLayer => Self::ByLayer,
            LineWeight::ByBlock => Self::ByBlock,
            LineWeight::Default => Self::Default,
            LineWeight::Value(v) => Self::Value(v),
        }
    }
    fn restore(&self) -> LineWeight {
        match *self {
            Self::ByLayer => LineWeight::ByLayer,
            Self::ByBlock => LineWeight::ByBlock,
            Self::Default => LineWeight::Default,
            Self::Value(v) => LineWeight::Value(v),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum SnapshotTransparency {
    ByLayer,
    ByBlock,
    Explicit(u8),
}
impl SnapshotTransparency {
    fn capture(v: Transparency) -> Self {
        match v {
            Transparency::ByLayer => Self::ByLayer,
            Transparency::ByBlock => Self::ByBlock,
            Transparency::Explicit(v) => Self::Explicit(v),
        }
    }
    fn restore(&self) -> Transparency {
        match *self {
            Self::ByLayer => Transparency::ByLayer,
            Self::ByBlock => Transparency::ByBlock,
            Self::Explicit(v) => Transparency::Explicit(v),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum SnapshotXDataValue {
    String(String),
    ControlString(String),
    LayerName(String),
    BinaryData(SnapshotBytes),
    Handle(SnapshotHandle),
    Point3D(SnapshotVector),
    Position3D(SnapshotVector),
    Displacement3D(SnapshotVector),
    Direction3D(SnapshotVector),
    Real(SnapshotFloat),
    Distance(SnapshotFloat),
    ScaleFactor(SnapshotFloat),
    Integer16(i16),
    Integer32(i32),
}
impl SnapshotXDataValue {
    fn capture(v: &XDataValue) -> Self {
        match v {
            XDataValue::String(v) => Self::String(v.clone()),
            XDataValue::ControlString(v) => Self::ControlString(v.clone()),
            XDataValue::LayerName(v) => Self::LayerName(v.clone()),
            XDataValue::BinaryData(v) => Self::BinaryData(SnapshotBytes(v.clone())),
            XDataValue::Handle(v) => Self::Handle(SnapshotHandle(*v)),
            XDataValue::Point3D(v) => Self::Point3D(SnapshotVector::capture(*v)),
            XDataValue::Position3D(v) => Self::Position3D(SnapshotVector::capture(*v)),
            XDataValue::Displacement3D(v) => Self::Displacement3D(SnapshotVector::capture(*v)),
            XDataValue::Direction3D(v) => Self::Direction3D(SnapshotVector::capture(*v)),
            XDataValue::Real(v) => Self::Real(SnapshotFloat(*v)),
            XDataValue::Distance(v) => Self::Distance(SnapshotFloat(*v)),
            XDataValue::ScaleFactor(v) => Self::ScaleFactor(SnapshotFloat(*v)),
            XDataValue::Integer16(v) => Self::Integer16(*v),
            XDataValue::Integer32(v) => Self::Integer32(*v),
        }
    }
    fn restore(&self) -> XDataValue {
        match self {
            Self::String(v) => XDataValue::String(v.clone()),
            Self::ControlString(v) => XDataValue::ControlString(v.clone()),
            Self::LayerName(v) => XDataValue::LayerName(v.clone()),
            Self::BinaryData(v) => XDataValue::BinaryData(v.0.clone()),
            Self::Handle(v) => XDataValue::Handle(v.0),
            Self::Point3D(v) => XDataValue::Point3D(v.restore()),
            Self::Position3D(v) => XDataValue::Position3D(v.restore()),
            Self::Displacement3D(v) => XDataValue::Displacement3D(v.restore()),
            Self::Direction3D(v) => XDataValue::Direction3D(v.restore()),
            Self::Real(v) => XDataValue::Real(v.0),
            Self::Distance(v) => XDataValue::Distance(v.0),
            Self::ScaleFactor(v) => XDataValue::ScaleFactor(v.0),
            Self::Integer16(v) => XDataValue::Integer16(*v),
            Self::Integer32(v) => XDataValue::Integer32(*v),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SnapshotXDataRecord {
    application_name: String,
    values: Vec<SnapshotXDataValue>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SnapshotExtendedData {
    records: Vec<SnapshotXDataRecord>,
    raw_dwg_eed: Vec<(SnapshotHandle, SnapshotBytes)>,
}
impl SnapshotExtendedData {
    fn capture(v: &ExtendedData) -> Self {
        Self {
            records: v
                .records()
                .iter()
                .map(|r| SnapshotXDataRecord {
                    application_name: r.application_name.clone(),
                    values: r.values.iter().map(SnapshotXDataValue::capture).collect(),
                })
                .collect(),
            raw_dwg_eed: v
                .raw_dwg_eed
                .iter()
                .map(|(handle, bytes)| {
                    (
                        SnapshotHandle(Handle::new(*handle)),
                        SnapshotBytes(bytes.clone()),
                    )
                })
                .collect(),
        }
    }
    fn restore(&self) -> ExtendedData {
        let mut v = ExtendedData::new();
        for r in &self.records {
            v.add_record(ExtendedDataRecord {
                application_name: r.application_name.clone(),
                values: r.values.iter().map(SnapshotXDataValue::restore).collect(),
            });
        }
        v.raw_dwg_eed = self
            .raw_dwg_eed
            .iter()
            .map(|(handle, bytes)| (handle.0.value(), bytes.0.clone()))
            .collect();
        v
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SplineCommonSnapshot {
    pub(crate) handle: SnapshotHandle,
    pub(crate) owner_handle: SnapshotHandle,
    layer: String,
    color: SnapshotColor,
    line_weight: SnapshotWeight,
    linetype: String,
    #[serde(deserialize_with = "required_option")]
    linetype_handle: Option<SnapshotHandle>,
    linetype_scale: SnapshotFloat,
    transparency: SnapshotTransparency,
    #[serde(deserialize_with = "required_option")]
    color_name: Option<String>,
    invisible: bool,
    extended_data: SnapshotExtendedData,
    #[serde(deserialize_with = "required_option")]
    graphic_data: Option<SnapshotBytes>,
    reactors: Vec<SnapshotHandle>,
    #[serde(deserialize_with = "required_option")]
    xdictionary_handle: Option<SnapshotHandle>,
    #[serde(deserialize_with = "required_option")]
    color_book_handle: Option<SnapshotHandle>,
    #[serde(deserialize_with = "required_option")]
    full_visual_style_handle: Option<SnapshotHandle>,
    #[serde(deserialize_with = "required_option")]
    face_visual_style_handle: Option<SnapshotHandle>,
    #[serde(deserialize_with = "required_option")]
    edge_visual_style_handle: Option<SnapshotHandle>,
    #[serde(deserialize_with = "required_option")]
    material_handle: Option<SnapshotHandle>,
    #[serde(deserialize_with = "required_option")]
    plotstyle_handle: Option<SnapshotHandle>,
    material_flags: u8,
    shadow_flags: u8,
    plotstyle_flags: u8,
    #[serde(deserialize_with = "required_option")]
    entity_mode: Option<u8>,
    has_ds_data: bool,
}
impl SplineCommonSnapshot {
    pub fn capture(common: &EntityCommon) -> Self {
        let EntityCommon {
            handle,
            owner_handle,
            layer,
            color,
            line_weight,
            linetype,
            linetype_handle,
            linetype_scale,
            transparency,
            color_name,
            invisible,
            extended_data,
            graphic_data,
            reactors,
            xdictionary_handle,
            color_book_handle,
            full_visual_style_handle,
            face_visual_style_handle,
            edge_visual_style_handle,
            material_handle,
            plotstyle_handle,
            material_flags,
            shadow_flags,
            plotstyle_flags,
            entity_mode,
            has_ds_data,
            raw_record: _,
        } = common;
        Self {
            handle: SnapshotHandle(*handle),
            owner_handle: SnapshotHandle(*owner_handle),
            layer: layer.clone(),
            color: SnapshotColor::capture(*color),
            line_weight: SnapshotWeight::capture(*line_weight),
            linetype: linetype.clone(),
            linetype_handle: linetype_handle.map(SnapshotHandle),
            linetype_scale: SnapshotFloat(*linetype_scale),
            transparency: SnapshotTransparency::capture(*transparency),
            color_name: color_name.clone(),
            invisible: *invisible,
            extended_data: SnapshotExtendedData::capture(extended_data),
            graphic_data: graphic_data.clone().map(SnapshotBytes),
            reactors: reactors.iter().copied().map(SnapshotHandle).collect(),
            xdictionary_handle: xdictionary_handle.map(SnapshotHandle),
            color_book_handle: color_book_handle.map(SnapshotHandle),
            full_visual_style_handle: full_visual_style_handle.map(SnapshotHandle),
            face_visual_style_handle: face_visual_style_handle.map(SnapshotHandle),
            edge_visual_style_handle: edge_visual_style_handle.map(SnapshotHandle),
            material_handle: material_handle.map(SnapshotHandle),
            plotstyle_handle: plotstyle_handle.map(SnapshotHandle),
            material_flags: *material_flags,
            shadow_flags: *shadow_flags,
            plotstyle_flags: *plotstyle_flags,
            entity_mode: *entity_mode,
            has_ds_data: *has_ds_data,
        }
    }
    pub fn restore(&self) -> EntityCommon {
        EntityCommon {
            handle: self.handle.0,
            owner_handle: self.owner_handle.0,
            layer: self.layer.clone(),
            color: self.color.restore(),
            line_weight: self.line_weight.restore(),
            linetype: self.linetype.clone(),
            linetype_handle: self.linetype_handle.map(|v| v.0),
            linetype_scale: self.linetype_scale.0,
            transparency: self.transparency.restore(),
            color_name: self.color_name.clone(),
            invisible: self.invisible,
            extended_data: self.extended_data.restore(),
            graphic_data: self.graphic_data.as_ref().map(|v| v.0.clone()),
            reactors: self.reactors.iter().map(|v| v.0).collect(),
            xdictionary_handle: self.xdictionary_handle.map(|v| v.0),
            color_book_handle: self.color_book_handle.map(|v| v.0),
            full_visual_style_handle: self.full_visual_style_handle.map(|v| v.0),
            face_visual_style_handle: self.face_visual_style_handle.map(|v| v.0),
            edge_visual_style_handle: self.edge_visual_style_handle.map(|v| v.0),
            material_handle: self.material_handle.map(|v| v.0),
            plotstyle_handle: self.plotstyle_handle.map(|v| v.0),
            material_flags: self.material_flags,
            shadow_flags: self.shadow_flags,
            plotstyle_flags: self.plotstyle_flags,
            entity_mode: self.entity_mode,
            has_ds_data: self.has_ds_data,
            raw_record: None,
        }
    }
}

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}
