use super::*;
use crate::geometry_kernel::hatch::{HatchBoundary2, HatchLineFamily};
record!(IfccadHatchContextPattern;0=>families:Vec<HatchLineFamily>=None,1=>scale:f64=None,2=>orientation_reference:[f64;3]=None,);
record!(IfccadHatchLoopClassification;
0=>external:bool=Some(false),
1=>polyline:bool=Some(false),
2=>derived:bool=Some(false),
3=>text_box:bool=Some(false),
4=>outermost:bool=Some(false),
5=>not_closed:bool=Some(false),
6=>self_intersecting:bool=Some(false),
7=>text_island:bool=Some(false),
8=>duplicate:bool=Some(false),
9=>is_annotative:bool=Some(false),
10=>does_not_support_scale:bool=Some(false),
11=>force_annotation_visible:bool=Some(false),
12=>orient_to_paper:bool=Some(false),
13=>is_annotative_block:bool=Some(false),
);
record!(IfccadHatchContextBoundary;0=>geometry:HatchBoundary2=None,1=>source_entity_ids:Vec<u64>=Some(Vec::new()),);
record!(IfccadHatchAnnotationContext;0=>pattern:IfccadHatchContextPattern=None,1=>loops:Vec<IfccadHatchContextLoop>=None,);
record!(IfccadHatchContextViewState;0=>normal:[f64;3]=None,1=>rotation:f64=None,2=>evaluate:bool=None,);
record!(IfccadHatchAnnotationViewContext;
0=>id:IfccadHatchAnnotationViewContextId=None,1=>scale_id:IfccadAnnotationScaleId=None,
2=>target:Option<IfccadAnnotationViewTarget>=Some(None),3=>presentation:IfccadHatchAnnotationContext=None,4=>view_state:IfccadHatchContextViewState=None,);
record!(IfccadHatchAnnotationViewContexts;
0=>next_context_id:u64=None,1=>reference:Option<IfccadHatchAnnotationViewContextId>=Some(None),2=>contexts:Vec<IfccadHatchAnnotationViewContext>=None,);
impl CborValue for IfccadHatchAnnotationViewContextId {
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
impl CborValue for IfccadAnnotationViewTarget {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "PaperViewport"), (1, "ModelWindow"), (2, "StoredView")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        let (kind, id) = match self {
            Self::PaperViewport { entity_id } => (0, *entity_id),
            Self::ModelWindow { id } => (1, id.0),
            Self::StoredView { id } => (2, id.0),
        };
        w.map(2)?;
        w.u64(0)?;
        w.u64(kind)?;
        w.u64(1)?;
        w.map(1)?;
        w.u64(0)?;
        id.encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let body = f.span(1)?;
        let mut b = RecordFields::new(&body, &[(0, "id")])?;
        let id: u64 = b.take(0, None)?;
        b.finish()?;
        f.finish()?;
        match code {
            0 => Ok(Self::PaperViewport { entity_id: id }),
            1 => Ok(Self::ModelWindow {
                id: IfccadModelWindowId(id),
            }),
            2 => Ok(Self::StoredView {
                id: IfccadAnnotationViewId(id),
            }),
            _ => Err(r.error("unknown annotation view target kind")),
        }
    }
    fn exact(&self, o: &Self) -> bool {
        self == o
    }
}
impl CborValue for IfccadHatchContextLoop {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Shared"), (1, "Local")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.map(2)?;
        w.u64(0)?;
        match self {
            Self::Shared {
                base_loop_index,
                classification,
            } => {
                w.u64(0)?;
                w.u64(1)?;
                w.map(2)?;
                w.u64(0)?;
                base_loop_index.encode(w)?;
                w.u64(1)?;
                classification.encode(w)
            }
            Self::Local {
                classification,
                boundary,
            } => {
                w.u64(1)?;
                w.u64(1)?;
                w.map(2)?;
                w.u64(0)?;
                classification.encode(w)?;
                w.u64(1)?;
                boundary.encode(w)
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let body = f.span(1)?;
        let out = match code {
            0 => {
                let mut b =
                    RecordFields::new(&body, &[(0, "base_loop_index"), (1, "classification")])?;
                let out = Self::Shared {
                    base_loop_index: b.take(0, None)?,
                    classification: b.take(1, None)?,
                };
                b.finish()?;
                out
            }
            1 => {
                let mut b = RecordFields::new(&body, &[(0, "classification"), (1, "boundary")])?;
                let out = Self::Local {
                    classification: b.take(0, None)?,
                    boundary: b.take(1, None)?,
                };
                b.finish()?;
                out
            }
            _ => return Err(r.error("unknown Hatch context boundary kind")),
        };
        f.finish()?;
        Ok(out)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::Shared {
                    base_loop_index: a,
                    classification: b,
                },
                Self::Shared {
                    base_loop_index: c,
                    classification: d,
                },
            ) => a.exact(c) && b.exact(d),
            (
                Self::Local {
                    classification: a,
                    boundary: b,
                },
                Self::Local {
                    classification: c,
                    boundary: d,
                },
            ) => a.exact(c) && b.exact(d),
            _ => false,
        }
    }
}
