use super::*;
record!(IfccadAttributeFlags;
0 => invisible: bool = Some(false),
1 => constant: bool = Some(false),
2 => verify: bool = Some(false),
3 => preset: bool = Some(false),
);
record!(IfccadAttributeMText;
0 => visible: bool = Some(true),
1 => layer_id: u64 = None,
2 => appearance: IfccadEntityAppearance = None,
3 => line_pattern_scale: f64 = Some(1.0),
4 => text: IfccadMText = None,
);
record!(IfccadBlockAttribute;
0 => id: u64 = None,
1 => visible: bool = Some(true),
2 => layer_id: u64 = None,
3 => appearance: IfccadEntityAppearance = None,
4 => line_pattern_scale: f64 = Some(1.0),
5 => value: IfccadAttributeValue = None,
);
record!(IfccadAttributeEmbeddedContext;
0 => scale_id: IfccadAnnotationScaleId = None,
1 => is_default: bool = Some(false),
2 => presentation: IfccadMTextAnnotationContext = None,
);
record!(IfccadAttributeFieldGraph;
0 => root: u64 = None,
1 => next_id: u64 = None,
2 => records: Vec<IfccadAttributeFieldRecord> = None,
);
record!(IfccadAttributeFieldRecord;
0 => id: u64 = None,
1 => evaluator: IfccadAttributeFieldEvaluator = None,
2 => code: Vec<IfccadAttributeFieldCodePart> = None,
3 => format: String = None,
4 => cache: IfccadAttributeFieldCache = None,
5 => evaluation_options: i32 = Some(0),
6 => filing_options: i32 = Some(0),
7 => state: i32 = Some(0),
8 => evaluation_status: i32 = Some(0),
9 => error_code: i32 = Some(0),
10 => error_message: String = None,
11 => source_evaluator: String = Some(String::new()),
12 => auxiliary_values: Vec<IfccadAttributeFieldAuxiliaryValue> = Some(Vec::new()),
);
record!(IfccadAttributeFieldCache;
0 => value: IfccadAttributeFieldValue = None,
1 => display: String = None,
2 => flags:i32=Some(0),
3 => unit_type:i32=Some(0),
4 => format:String=Some(String::new()),
5 => formatted_value:String=Some(String::new()),
);
record!(IfccadAttributeDefinition;
0 => tag: String = None,
1 => prompt: String = None,
2 => flags: IfccadAttributeFlags = Some(Default::default()),
3 => lock_position: bool = Some(false),
4 => field_length: i32 = Some(0),
5 => text: IfccadText = None,
6 => m_text: Option<Box<IfccadAttributeMText>> = Some(None),
7 => annotation: Option<Box<IfccadAnnotation<IfccadAttributeAnnotationContext>>> = Some(None),
8 => secondary_attributes: Vec<u64> = Some(Vec::new()),
9 => fields: Option<IfccadAttributeFieldGraph> = Some(None),
);
record!(IfccadAttributeValue;
0 => tag: String = None,
1 => definition_id: Option<u64> = Some(None),
2 => flags: IfccadAttributeFlags = Some(Default::default()),
3 => lock_position: bool = Some(false),
4 => field_length: i32 = Some(0),
5 => text: IfccadText = None,
6 => m_text: Option<Box<IfccadAttributeMText>> = Some(None),
7 => annotation: Option<Box<IfccadAnnotation<IfccadAttributeAnnotationContext>>> = Some(None),
8 => secondary_attributes: Vec<u64> = Some(Vec::new()),
9 => fields: Option<IfccadAttributeFieldGraph> = Some(None),
);
unit_enum!(IfccadAttributeVisibility;0=>Normal,1=>HideAll,2=>ShowAll);
unit_enum!(IfccadAttributeFieldEvaluator;0=>TextTemplate,1=>DrawingVariable,2=>ObjectProperty);
impl CborValue for IfccadAttributeAnnotationContext {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Text"), (1, "MText")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Text { text } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                text.encode(w)?;
                Ok(())
            }
            Self::MText {
                text,
                m_text_enabled,
                embedded,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                w.map(3)?;
                w.u64(0)?;
                text.encode(w)?;
                w.u64(1)?;
                m_text_enabled.encode(w)?;
                w.u64(2)?;
                embedded.encode(w)?;
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let tag: u64 = f.take(0, None)?;
        let body = f.span(1)?;
        f.finish()?;
        match tag {
            0 => {
                let mut b = RecordFields::new(&body, &[(0, "text")])?;
                let text: IfccadTextAnnotationContext = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Text { text })
            }
            1 => {
                let mut b = RecordFields::new(
                    &body,
                    &[(0, "text"), (1, "m_text_enabled"), (2, "embedded")],
                )?;
                let text: IfccadTextAnnotationContext = b.take(0, None)?;
                let m_text_enabled: bool = b.take(1, None)?;
                let embedded: Option<IfccadAttributeEmbeddedContext> = b.take(2, Some(None))?;
                b.finish()?;
                Ok(Self::MText {
                    text,
                    m_text_enabled,
                    embedded,
                })
            }
            v => Err(r.error(format!("unknown attribute variant {v}"))),
        }
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Text { text }, Self::Text { text: text_o }) => text.exact(text_o),
            (
                Self::MText {
                    text,
                    m_text_enabled,
                    embedded,
                },
                Self::MText {
                    text: text_o,
                    m_text_enabled: m_text_enabled_o,
                    embedded: embedded_o,
                },
            ) => {
                text.exact(text_o)
                    && m_text_enabled.exact(m_text_enabled_o)
                    && embedded.exact(embedded_o)
            }
            _ => false,
        }
    }
}
impl CborValue for IfccadAttributeFieldCodePart {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "Literal"),
            (1, "Child"),
            (2, "Object"),
            (3, "CurrentBlockReference"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Literal(v) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                v.encode(w)?;
                Ok(())
            }
            Self::Child(v) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                v.encode(w)?;
                Ok(())
            }
            Self::Object(v) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                v.encode(w)?;
                Ok(())
            }
            Self::CurrentBlockReference => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(3)?;
                w.u64(1)?;
                w.map(0)?;
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let tag: u64 = f.take(0, None)?;
        let body = f.span(1)?;
        f.finish()?;
        match tag {
            0 => {
                let mut b = RecordFields::new(&body, &[(0, "v")])?;
                let v: String = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Literal(v))
            }
            1 => {
                let mut b = RecordFields::new(&body, &[(0, "v")])?;
                let v: u64 = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Child(v))
            }
            2 => {
                let mut b = RecordFields::new(&body, &[(0, "v")])?;
                let v: u64 = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Object(v))
            }
            3 => {
                let b = RecordFields::new(&body, &[])?;
                b.finish()?;
                Ok(Self::CurrentBlockReference)
            }
            v => Err(r.error(format!("unknown attribute variant {v}"))),
        }
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Literal(v), Self::Literal(v_o)) => v.exact(v_o),
            (Self::Child(v), Self::Child(v_o)) => v.exact(v_o),
            (Self::Object(v), Self::Object(v_o)) => v.exact(v_o),
            (Self::CurrentBlockReference, Self::CurrentBlockReference) => true,
            _ => false,
        }
    }
}
impl CborValue for IfccadAttributeFieldValue {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "Empty"),
            (1, "Text"),
            (2, "Integer"),
            (3, "Real"),
            (4, "Boolean"),
            (5, "Object"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Empty => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                w.map(0)?;
                Ok(())
            }
            Self::Text(v) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                v.encode(w)?;
                Ok(())
            }
            Self::Integer(v) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                v.encode(w)?;
                Ok(())
            }
            Self::Real(v) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(3)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                v.encode(w)?;
                Ok(())
            }
            Self::Object(v) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(5)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                v.encode(w)?;
                Ok(())
            }
            Self::Boolean(v) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(4)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                v.encode(w)?;
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let tag: u64 = f.take(0, None)?;
        let body = f.span(1)?;
        f.finish()?;
        match tag {
            0 => {
                let b = RecordFields::new(&body, &[])?;
                b.finish()?;
                Ok(Self::Empty)
            }
            1 => {
                let mut b = RecordFields::new(&body, &[(0, "v")])?;
                let v: String = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Text(v))
            }
            2 => {
                let mut b = RecordFields::new(&body, &[(0, "v")])?;
                let v: i64 = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Integer(v))
            }
            3 => {
                let mut b = RecordFields::new(&body, &[(0, "v")])?;
                let v: f64 = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Real(v))
            }
            4 => {
                let mut b = RecordFields::new(&body, &[(0, "v")])?;
                let v: bool = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Boolean(v))
            }
            5 => {
                let mut b = RecordFields::new(&body, &[(0, "v")])?;
                let v: u64 = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Object(v))
            }
            v => Err(r.error(format!("unknown attribute variant {v}"))),
        }
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Empty, Self::Empty) => true,
            (Self::Text(v), Self::Text(v_o)) => v.exact(v_o),
            (Self::Integer(v), Self::Integer(v_o)) => v.exact(v_o),
            (Self::Real(v), Self::Real(v_o)) => v.exact(v_o),
            (Self::Object(v), Self::Object(v_o)) => v.exact(v_o),
            (Self::Boolean(v), Self::Boolean(v_o)) => v.exact(v_o),
            _ => false,
        }
    }
}

record!(IfccadAttributeFieldAuxiliaryValue;0=>key:String=None,1=>cache:IfccadAttributeFieldCache=None,);
