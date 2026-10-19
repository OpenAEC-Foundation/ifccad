use super::*;

// A present opaque appearance has known, complete modes. Native field omission
// does not apply inside this envelope; an absent envelope still means unknown.
impl CborValue for IfccadOpaqueAppearance {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "appearance"), (1, "line_pattern_scale")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        let Self {
            appearance,
            line_pattern_scale,
        } = self;
        let IfccadEntityAppearance {
            color,
            opacity,
            line_pattern,
            line_weight,
        } = appearance;
        w.map(2)?;
        w.u64(0)?;
        w.map(4)?;
        w.u64(0)?;
        color.encode(w)?;
        w.u64(1)?;
        opacity.encode(w)?;
        w.u64(2)?;
        line_pattern.encode(w)?;
        w.u64(3)?;
        line_weight.encode(w)?;
        w.u64(1)?;
        line_pattern_scale.encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut fields = RecordFields::new(r, &[(0, "appearance"), (1, "line_pattern_scale")])?;
        let body = fields.span(0)?;
        let mut modes = RecordFields::new(
            &body,
            &[
                (0, "color"),
                (1, "opacity"),
                (2, "line_pattern"),
                (3, "line_weight"),
            ],
        )?;
        let appearance = IfccadEntityAppearance {
            color: modes.take(0, None)?,
            opacity: modes.take(1, None)?,
            line_pattern: modes.take(2, None)?,
            line_weight: modes.take(3, None)?,
        };
        modes.finish()?;
        let result = Self {
            appearance,
            line_pattern_scale: fields.take(1, None)?,
        };
        fields.finish()?;
        Ok(result)
    }
    fn exact(&self, o: &Self) -> bool {
        let Self {
            appearance,
            line_pattern_scale,
        } = self;
        appearance.exact(&o.appearance) && line_pattern_scale.exact(&o.line_pattern_scale)
    }
}
record!(IfccadPreservation;
0 => version: u32 = None,
1 => sources: Vec<IfccadPreservationSource> = None,
2 => records: Vec<IfccadPreservationRecord> = None,
);
record!(IfccadPreservationSource;
0 => id: String = None,
1 => provider: String = None,
2 => provider_revision: String = None,
3 => origin: IfccadPreservationOrigin = None,
4 => source_version: Option<String> = Some(None),
);
record!(IfccadPreservationRecord;
0 => id: IfccadPreservationRecordId = None,
1 => source_id: String = None,
2 => source_key: String = None,
3 => category: IfccadPreservationCategory = None,
4 => role: IfccadPreservationRole = None,
5 => representation: IfccadPreservationRepresentation = None,
6 => subject: Option<IfccadPreservationTarget> = Some(None),
7 => dependency_coverage: IfccadPreservationDependencyCoverage = None,
8 => bindings: Vec<IfccadPreservationBinding> = None,
9 => conditions: Vec<IfccadPreservationCondition> = None,
10 => payload: IfccadPreservationPayload = None,
);
impl CborValue for IfccadPreservationRecordId {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        self.0.encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        Ok(Self(CborValue::decode(r)?))
    }
    fn exact(&self, o: &Self) -> bool {
        self.0.exact(&o.0)
    }
}
unit_enum!(IfccadPreservationOrigin;
0 => CadDocument,
1 => Dwg,
2 => Dxf,
);
unit_enum!(IfccadPreservationCategory;
0 => Entity,
1 => Object,
2 => Table,
3 => Drawing,
4 => Layout,
5 => Shared,
);
unit_enum!(IfccadPreservationRole;
0 => Complete,
1 => Supplement,
2 => Shared,
);
unit_enum!(IfccadPreservationRepresentation;
0 => CodecTyped,
1 => CodecOpaque,
);
unit_enum!(IfccadPreservationDependencyCoverage;
0 => Qualified,
1 => Conservative,
2 => Unknown,
);
record!(IfccadPreservationPayload;
0 => schema: String = None,
1 => version: u32 = None,
2 => kind: IfccadPreservationPayloadKind = None,
3 => bytes: Vec<u8> = None,
);
impl CborValue for IfccadPreservationTarget {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "Drawing"),
            (1, "Entity"),
            (2, "Layer"),
            (3, "LinePattern"),
            (4, "Layout"),
            (5, "BlockDefinition"),
            (6, "Record"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Drawing => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::Entity(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<u64>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<u64>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::Layer(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<u64>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<u64>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::LinePattern(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(3)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<IfccadLinePatternId>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<IfccadLinePatternId>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::Layout(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(4)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<u64>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<u64>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::BlockDefinition(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(5)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<u64>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<u64>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::Record(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(6)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<IfccadPreservationRecordId>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<IfccadPreservationRecordId>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => Self::Drawing,
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: u64 = b.take(0, None)?;
                b.finish()?;
                Self::Entity(value_0)
            }
            2 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: u64 = b.take(0, None)?;
                b.finish()?;
                Self::Layer(value_0)
            }
            3 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: IfccadLinePatternId = b.take(0, None)?;
                b.finish()?;
                Self::LinePattern(value_0)
            }
            4 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: u64 = b.take(0, None)?;
                b.finish()?;
                Self::Layout(value_0)
            }
            5 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: u64 = b.take(0, None)?;
                b.finish()?;
                Self::BlockDefinition(value_0)
            }
            6 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: IfccadPreservationRecordId = b.take(0, None)?;
                b.finish()?;
                Self::Record(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Drawing, Self::Drawing) => true,
            (Self::Entity(value_0), Self::Entity(value_0_other)) => value_0.exact(value_0_other),
            (Self::Layer(value_0), Self::Layer(value_0_other)) => value_0.exact(value_0_other),
            (Self::LinePattern(value_0), Self::LinePattern(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::Layout(value_0), Self::Layout(value_0_other)) => value_0.exact(value_0_other),
            (Self::BlockDefinition(value_0), Self::BlockDefinition(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::Record(value_0), Self::Record(value_0_other)) => value_0.exact(value_0_other),
            _ => false,
        }
    }
}
record!(IfccadPreservationBinding;
0 => slot: String = None,
1 => source_key: String = None,
2 => target: IfccadPreservationTarget = None,
);
record!(IfccadPreservationCondition;
0 => target: IfccadPreservationTarget = None,
1 => predicate: String = None,
2 => version: u32 = None,
3 => baseline: Vec<u8> = None,
);
unit_enum!(IfccadPreservationPayloadKind;
0 => AdapterSnapshot,
1 => RawDwgRecord,
2 => RawDxfGroups,
3 => RawSection,
);
