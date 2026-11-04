use super::*;

impl CborValue for IfccadAnnotationScaleId {
    fn encode(&self, writer: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        self.0.encode(writer)
    }
    fn decode(reader: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        Ok(Self(CborValue::decode(reader)?))
    }
    fn exact(&self, other: &Self) -> bool {
        self == other
    }
}

record!(IfccadAnnotationScale;
0 => id: IfccadAnnotationScaleId = None,
1 => name: String = None,
2 => paper_units: f64 = None,
3 => drawing_units: f64 = None,
4 => unit_scale: bool = Some(false),
);

unit_enum!(IfccadAnnotationMode; 0 => Disabled, 1 => Enabled, 2 => ByDefinition);
record!(IfccadAnnotationContextKey; 0 => scale_id: IfccadAnnotationScaleId = None,);
impl<P: CborValue> CborValue for IfccadAnnotation<P> {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.map(2 + u64::from(self.reference.is_some()) + u64::from(self.orientation.is_some()))?;
        w.u64(0)?;
        self.mode.encode(w)?;
        if self.reference.is_some() {
            w.u64(1)?;
            self.reference.encode(w)?;
        }
        w.u64(2)?;
        self.contexts.encode(w)?;
        if self.orientation.is_some() {
            w.u64(3)?;
            self.orientation.encode(w)?;
        }
        Ok(())
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(
            r,
            &[
                (0, "mode"),
                (1, "reference"),
                (2, "contexts"),
                (3, "orientation"),
            ],
        )?;
        let a = Self {
            orientation: f.take(3, Some(None))?,
            mode: f.take(0, None)?,
            reference: f.take(1, Some(None))?,
            contexts: f.take(2, None)?,
        };
        f.finish()?;
        Ok(a)
    }
    fn exact(&self, o: &Self) -> bool {
        self.orientation.exact(&o.orientation)
            && self.mode.exact(&o.mode)
            && self.reference.exact(&o.reference)
            && self.contexts.exact(&o.contexts)
    }
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "mode"),
            (1, "reference"),
            (2, "contexts"),
            (3, "orientation"),
        ]
    }
}
impl<P: CborValue> CborValue for IfccadAnnotationContext<P> {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.map(if self.presentation.is_some() { 2 } else { 1 })?;
        w.u64(0)?;
        self.key.encode(w)?;
        if self.presentation.is_some() {
            w.u64(1)?;
            self.presentation.encode(w)?;
        }
        Ok(())
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "key"), (1, "presentation")])?;
        let c = Self {
            key: f.take(0, None)?,
            presentation: f.take(1, Some(None))?,
        };
        f.finish()?;
        Ok(c)
    }
    fn exact(&self, o: &Self) -> bool {
        self.key.exact(&o.key) && self.presentation.exact(&o.presentation)
    }
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "key"), (1, "presentation")]
    }
}
unit_enum!(IfccadTextAnnotationHorizontal;0=>Left,1=>Center,2=>Right,3=>Aligned,4=>Middle,5=>Fit);
unit_enum!(IfccadAnnotationColumnKind;0=>Static,1=>Dynamic);
record!(IfccadTextAnnotationContext;
0=>insertion:[f64;2]=None,1=>alignment:[f64;2]=None,2=>rotation:f64=None,3=>horizontal:IfccadTextAnnotationHorizontal=None,);
record!(IfccadMTextAnnotationColumns;
0=>kind:IfccadAnnotationColumnKind=None,1=>count:u32=None,2=>width:f64=None,3=>gutter:f64=None,
4=>auto_height:bool=None,5=>flow_reversed:bool=None,6=>heights:Vec<f64>=None,);
record!(IfccadMTextAnnotationContext;
0=>insertion:[f64;3]=None,1=>x_direction:[f64;3]=None,2=>attachment:crate::text::MTextAttachment=None,
3=>rectangle_width:f64=None,4=>rectangle_height:f64=None,5=>columns:Option<IfccadMTextAnnotationColumns>=Some(None),);

unit_enum!(IfccadAnnotationOrientation;0=>Fixed,1=>MatchLayout);
record!(IfccadAnnotationDefinition;
0=>enabled:bool=Some(false),1=>orientation:IfccadAnnotationOrientation=Some(IfccadAnnotationOrientation::Fixed),);

impl CborValue for IfccadAnnotationViewId {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        self.0.encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        Ok(Self(u64::decode(r)?))
    }
    fn exact(&self, o: &Self) -> bool {
        self == o
    }
}
unit_enum!(IfccadAnnotationViewSpace;0=>Model,1=>Paper);
record!(IfccadAnnotationStoredView;
0=>id:IfccadAnnotationViewId=None,1=>name:String=None,2=>space:IfccadAnnotationViewSpace=None,
3=>width:f64=None,4=>view:IfccadViewportView=None,5=>annotation_scale_id:Option<IfccadAnnotationScaleId>=Some(None),);

record!(IfccadBlockAnnotationContext;0=>transform:IfccadBlockTransform=None,);

unit_enum!(IfccadModelLinetypeAnnotationMode;0=>Unscaled,1=>ByAnnotationScale);
unit_enum!(IfccadAnnotationVisibility;0=>CurrentScaleOnly,1=>All);
record!(IfccadDrawingAnnotationSettings;0=>insert_as_annotative:Option<bool>=Some(None),1=>model_linetype_scaling:Option<IfccadModelLinetypeAnnotationMode>=Some(None),);
record!(IfccadLayoutAnnotationState;0=>current_scale_id:Option<IfccadAnnotationScaleId>=Some(None),1=>visibility:Option<IfccadAnnotationVisibility>=Some(None),);
