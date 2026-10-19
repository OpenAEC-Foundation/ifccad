use super::*;
use crate::text::{
    CharacterFormat, FontRequest, MTextAttachment, MTextBackground, MTextColumns, MTextFlow,
    MTextParagraph, ParagraphFormat, TextLayout, TextRun, TextStyleProperties,
};
record!(IfccadTextStyle;
0 => id: IfccadTextStyleId = None,
1 => name: String = None,
2 => properties: TextStyleProperties = None,
);
impl CborValue for IfccadTextStyleId {
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
record!(TextStyleProperties;
0 => font: FontRequest = None,
1 => width_factor: f64 = Some(1.0),
2 => oblique_angle: f64 = Some(0.0),
3 => vertical: bool = Some(false),
4 => creation_height: Option<f64> = Some(None),
5 => last_used_height: Option<f64> = Some(None),
6 => creation_backward: bool = Some(false),
7 => creation_upside_down: bool = Some(false),
);
record!(FontRequest;
0 => family: Option<String> = Some(None),
1 => cad_font_name: Option<String> = Some(None),
2 => big_font_name: Option<String> = Some(None),
3 => bold: Option<bool> = Some(None),
4 => italic: Option<bool> = Some(None),
5 => charset: Option<u16> = Some(None),
6 => pitch: Option<u8> = Some(None),
);
record!(IfccadText;
0 => style_id: IfccadTextStyleId = None,
1 => placement: IfccadPlacement = None,
2 => rotation: f64 = Some(0.0),
3 => backward: bool = Some(false),
4 => upside_down: bool = Some(false),
5 => layout: TextLayout = None,
6 => oblique_angle: f64 = Some(0.0),
7 => thickness: f64 = Some(0.0),
8 => content: Vec<TextRun> = None,
);
record!(IfccadMText;
0 => style_id: IfccadTextStyleId = None,
1 => placement: IfccadPlacement = None,
2 => rotation: f64 = Some(0.0),
3 => backward: bool = Some(false),
4 => upside_down: bool = Some(false),
5 => height: f64 = None,
6 => attachment: MTextAttachment = None,
7 => flow: MTextFlow = None,
8 => wrap_width: Option<f64> = Some(None),
9 => columns: Option<MTextColumns> = Some(None),
10 => background: Option<MTextBackground<IfccadColor>> = Some(None),
11 => character_format: CharacterFormat<IfccadColor> = None,
12 => paragraph_format: ParagraphFormat = None,
13 => content: Vec<MTextParagraph<IfccadColor>> = None,
);
