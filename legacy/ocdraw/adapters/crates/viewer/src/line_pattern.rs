//! Inspection values preserve marker identities independently of native storage.
use ocdraw::{line_pattern::*, ocdraw::*};
use serde_json::{json, Value};

pub(crate) fn ocdraw_elements(elements: &[DrawingLinePatternElement]) -> Value {
    json!(elements.iter().map(|e| {
        let Some(marker)=&e.marker else {return json!(e.length);};
        let (mut m,t)=match marker {
            DrawingLinePatternMarker::Text {text,text_style_id,transform}=>(json!({"kind":"text","text":text,"textStyleId":text_style_id.0}),transform),
            DrawingLinePatternMarker::Symbol {symbol_source_id,shape_number,transform}=>(json!({"kind":"symbol","symbolSourceId":symbol_source_id.0,"shapeNumber":shape_number}),transform),
        };
        m["scale"]=json!(t.scale);m["offset"]=json!(t.offset);
        m["rotation"]=match t.rotation {
            LinePatternRotation::Relative(angle)=>json!({"mode":"relative","angle":angle}),
            LinePatternRotation::Absolute(angle)=>json!({"mode":"absolute","angle":angle}),
        };
        json!({"length":e.length,"marker":m})
    }).collect::<Vec<_>>())
}

pub(crate) fn ocdraw_sources(sources: &[DrawingSymbolSource]) -> Value {
    json!(sources
        .iter()
        .map(|s| match &s.source {
            SymbolSource::Shx { file_name } =>
                json!({"id":s.id.0,"kind":"shx","fileName":file_name}),
        })
        .collect::<Vec<_>>())
}
