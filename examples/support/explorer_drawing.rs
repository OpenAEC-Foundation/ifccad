//! Small authoring helpers for the two Explorer drawings, outside the core API.
#![allow(clippy::too_many_arguments)] // Coordinate arguments keep the authoring recipes legible.
use serde_json::{json, Value};
use std::error::Error;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub const MODEL: usize = 1;

pub fn plane(x: f64, y: f64) -> Value {
    json!({"origin":[x,y,0.],"xAxis":[1.,0.,0.],"yAxis":[0.,1.,0.]})
}

pub struct Drawing {
    pub nodes: Vec<Value>,
    next_entity: u64,
    next_layer: u64,
    next_block: u64,
}

impl Drawing {
    pub fn new(id: &str) -> Self {
        let mut d = Self {
            nodes: vec![
                json!({"path":"/cad/d1","children":{"model":"/cad/d1/layout/1","continuous":"/cad/d1/linePattern/0","text":"/cad/d1/textStyle/0","dimensions":"/cad/d1/dimensionStyle/0"},"attributes":{"ifccad::drawing":{"profileVersion":"0.1.0","lengthUnit":"mm","nextEntityId":1,"nextLayerId":1,"nextLayoutId":2,"nextBlockId":1,"nextLinePatternId":1,"nextTextStyleId":1,"nextDimensionStyleId":1,"nextUcsId":1,"nextModelWindowId":1,"nextSymbolSourceId":1}}}),
                json!({"path":"/cad/d1/layout/1","children":{},"attributes":{"ifccad::layout":{"kind":"Model","tabIndex":0}}}),
                json!({"path":"/cad/d1/linePattern/0","attributes":{"ifccad::linePattern":{"name":"Continuous","pattern":[]}}}),
                json!({"path":"/cad/d1/textStyle/0","attributes":{"ifccad::textStyle":{"name":"Drawing labels","font":{"cadFontName":"Arial"}}}}),
            ],
            next_entity: 1,
            next_layer: 0,
            next_block: 1,
        };
        // Reuse the qualified ordinary style; the catalogue does not implement a renderer.
        let styles: Value =
            serde_json::from_slice(include_bytes!("../ifccad/dimensions.ifcx")).unwrap();
        let mut style = styles["data"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["attributes"].get("ifccad::dimensionStyle").is_some())
            .unwrap()
            .clone();
        style["attributes"]["ifccad::dimensionStyle"]["typeOverrides"] = json!({});
        let settings = &mut style["attributes"]["ifccad::dimensionStyle"];
        settings["name"] = json!("Millimetres");
        settings["settings"]["numbers"]["primary"]["precision"] = json!(2);
        settings["settings"]["numbers"]["primary"]["suffix"] = json!("");
        settings["settings"]["numbers"]["primary"]["zeroSuppression"]["trailing"] = json!(true);
        d.nodes.push(style);
        d.nodes[0]["attributes"]["example::id"] = json!(id);
        d.layer("0", None, 0.18);
        d
    }

    pub fn layer(&mut self, name: &str, rgb: Option<[u8; 3]>, weight: f64) -> u64 {
        let id = self.next_layer;
        self.next_layer += 1;
        let path = format!("/cad/d1/layer/{id}");
        self.nodes[0]["children"][format!("layer{id}")] = json!(path);
        let color = rgb.map_or(json!({"kind":"Foreground"}), |rgb| json!({"rgb":rgb}));
        self.nodes.push(json!({"path":path,"attributes":{"ifccad::layer":{"name":name,"appearance":{"color":color,"linePattern":"/cad/d1/linePattern/0","lineWeight":weight,"opacity":1.}}}}));
        id
    }

    pub fn pattern(&mut self) {
        self.nodes[0]["children"]["dashed"] = json!("/cad/d1/linePattern/1");
        self.nodes[0]["attributes"]["ifccad::drawing"]["nextLinePatternId"] = json!(2);
        self.nodes.push(json!({"path":"/cad/d1/linePattern/1","attributes":{"ifccad::linePattern":{"name":"Dashed","pattern":[8.,-4.]}}}));
    }

    pub fn entity(&mut self, owner: usize, layer: u64, mut attributes: Value) -> u64 {
        let id = self.next_entity;
        self.next_entity += 1;
        let path = format!("/cad/d1/e{id}");
        let n = self.nodes[owner]["children"].as_object().unwrap().len();
        self.nodes[owner]["children"][n.to_string()] = json!(path);
        attributes["ifccad::entity"] = json!({"layer":format!("/cad/d1/layer/{layer}"),"appearance":{"color":{"mode":"ByLayer"},"linePattern":{"mode":"ByLayer"},"lineWeight":{"mode":"ByLayer"},"opacity":{"mode":"ByLayer"}}});
        self.nodes
            .push(json!({"path":path,"attributes":attributes}));
        id
    }

    pub fn line(&mut self, owner: usize, layer: u64, a: [f64; 2], b: [f64; 2]) -> u64 {
        self.entity(
            owner,
            layer,
            json!({"ifccad::geom::lineSegment":{"start":[a[0],a[1],0.],"end":[b[0],b[1],0.]}}),
        )
    }
    pub fn rect(&mut self, owner: usize, layer: u64, x: f64, y: f64, w: f64, h: f64) {
        self.poly(
            owner,
            layer,
            vec![[x, y], [x + w, y], [x + w, y + h], [x, y + h]],
            true,
        );
    }
    pub fn poly(&mut self, owner: usize, layer: u64, vertices: Vec<[f64; 2]>, closed: bool) {
        self.entity(owner,layer,json!({"ifccad::geom::placement":plane(0.,0.),"ifccad::geom::planarPolyline":{"vertices":vertices,"closed":closed}}));
    }
    pub fn text(&mut self, owner: usize, layer: u64, x: f64, y: f64, h: f64, text: &str) {
        self.entity(owner,layer,json!({"ifccad::geom::placement":plane(x,y),"ifccad::text":{"style":"/cad/d1/textStyle/0","content":[{"text":text}],"layout":{"kind":"anchored","height":h,"horizontal":"left","vertical":"baseline","widthFactor":1.}}}));
    }
    pub fn circle(&mut self, owner: usize, layer: u64, x: f64, y: f64, r: f64) {
        self.entity(
            owner,
            layer,
            json!({"ifccad::geom::placement":plane(x,y),"ifccad::geom::circle":{"radius":r}}),
        );
    }
    pub fn arc(
        &mut self,
        owner: usize,
        layer: u64,
        x: f64,
        y: f64,
        r: f64,
        start: f64,
        sweep: f64,
    ) {
        self.entity(owner,layer,json!({"ifccad::geom::placement":plane(x,y),"ifccad::geom::arc":{"radius":r,"startParameter":start,"sweepParameter":sweep}}));
    }
    pub fn hatch(
        &mut self,
        layer: u64,
        vertices: Vec<[f64; 2]>,
        fill: Value,
        gradient: Option<Value>,
    ) {
        let n = vertices.len();
        let mut hatch = json!({"placement":null,"areaRule":"normal","fill":fill,"joinTolerance":1e-9,"associationEnabled":false,"loops":[{"boundary":{"kind":"polyline","vertices":vertices,"bulges":vec![0.;n]}}]});
        hatch.as_object_mut().unwrap().remove("placement");
        if let Some(gradient) = gradient {
            hatch["gradient"] = gradient;
        }
        self.entity(
            MODEL,
            layer,
            json!({"ifccad::geom::placement":plane(0.,0.),"ifccad::hatch":hatch}),
        );
    }
    pub fn block(&mut self, name: &str) -> (usize, u64) {
        let id = self.next_block;
        self.next_block += 1;
        let path = format!("/cad/d1/block/{id}");
        self.nodes[0]["children"][format!("block{id}")] = json!(path);
        let owner = self.nodes.len();
        self.nodes.push(json!({"path":path,"children":{},"attributes":{"ifccad::blockDefinition":{"name":name,"basePoint":[0.,0.,0.],"insertionUnit":"mm"}}}));
        (owner, id)
    }
    pub fn insert(
        &mut self,
        layer: u64,
        block: u64,
        x: f64,
        y: f64,
        rotation: f64,
        scale: [f64; 3],
    ) {
        self.entity(MODEL,layer,json!({"ifccad::blockInstance":{"definition":format!("/cad/d1/block/{block}"),"transform":{"placement":plane(x,y),"rotation":rotation,"scale":scale}}}));
    }
    pub fn dimension(
        &mut self,
        layer: u64,
        a: [f64; 2],
        b: [f64; 2],
        p: [f64; 2],
        aligned: bool,
        height: f64,
    ) {
        let mut definition = json!({"kind":if aligned {"aligned"} else {"linear"},"firstPoint":a,"secondPoint":b,"dimensionLinePoint":p});
        if !aligned {
            definition["directionAngle"] = json!(if a[0] == b[0] {
                std::f64::consts::FRAC_PI_2
            } else {
                0.
            });
        }
        self.entity(MODEL,layer,json!({"ifccad::geom::placement":plane(0.,0.),"ifccad::dimension":{"style":"/cad/d1/dimensionStyle/0","definition":definition,"label":{"kind":"generated"},"textPlacement":{"kind":"styleDriven"},"overrides":{"text":{"height":height,"gap":height*0.25},"arrows":{"size":height},"extensionLines":{"first":{"extension":height*0.5,"offset":height*0.25},"second":{"extension":height*0.5,"offset":height*0.25}}},"associations":[]}}));
    }
    pub fn dimension_precision(&mut self, precision: u8) {
        let style = self
            .nodes
            .iter_mut()
            .find(|n| n["attributes"].get("ifccad::dimensionStyle").is_some())
            .unwrap();
        style["attributes"]["ifccad::dimensionStyle"]["settings"]["numbers"]["primary"]
            ["precision"] = json!(precision);
    }
    pub fn sheet(
        &mut self,
        title: &str,
        center: [f64; 2],
        view_height: f64,
        width: f64,
        height: f64,
    ) -> usize {
        self.nodes[0]["children"]["paper"] = json!("/cad/d1/layout/2");
        self.nodes[0]["attributes"]["ifccad::drawing"]["nextLayoutId"] = json!(3);
        let paper = self.nodes.len();
        self.nodes.push(json!({"path":"/cad/d1/layout/2","children":{},"attributes":{"ifccad::layout":{"kind":"Paper","name":title,"tabIndex":1,"media":{"width":420.,"height":297.,"unit":"mm"},"plotSettings":{"plotUnit":"mm","page":{"printableArea":{"minX":0.,"minY":0.,"maxX":420.,"maxY":297.},"rotation":"none"},"area":{"mode":"Layout"},"mapping":{"scale":{"mode":"Fixed","outputLength":1.,"scopeLength":1.},"placement":{"mode":"Offset","reference":"Media","x":0.,"y":0.}},"output":{"shadedPlot":{"mode":"AsDisplayed","quality":{"mode":"Normal"}},"applyPlotStyles":false},"options":{"plotViewportBorders":false,"plotPaperSpaceLast":false,"hidePaperSpaceObjects":false,"plotLineWeights":true,"scaleLineWeights":false,"plotTransparency":false}}}}}));
        self.entity(paper,0,json!({"ifccad::viewport":{"model":"/cad/d1/layout/1","frame":{"center":[210.,158.],"width":width,"height":height},"view":{"target":[center[0],center[1],0.],"center":[0.,0.],"direction":[0.,0.,1.],"height":view_height,"twist":0.,"projection":"Orthographic","lensLengthMm":50.,"frontClip":{"mode":"Disabled"},"backClip":{"mode":"Disabled"}},"viewEnabled":true,"viewLocked":true,"renderMode":"TwoDimensional","paperClip":{"enabled":false},"layerOverrides":[]}}));
        self.nodes[paper]["attributes"]["ifccad::paperCanvas"] = json!({
            "storedUcs":{"kind":"World"},"useStoredUcs":false,
            "grid":{"enabled":false,"spacing":[10.,10.],"style":"Lines","majorLineFrequency":5,"beyondLimits":false,"adaptive":false,"subdivision":false,"followsWorkplane":false},
            "snap":{"enabled":false,"base":[0.,0.],"spacing":[10.,10.],"angle":0.,"style":"Rectangular","isometricPlane":"Top"},
            "view":{"center":[210.,148.5],"target":[0.,0.,0.],"direction":[0.,0.,1.],"height":350.,"twist":0.,"projection":"Orthographic","frontClip":{"mode":"Disabled"},"backClip":{"mode":"Disabled"}}
        });
        paper
    }
    pub fn model_view(&mut self, center: [f64; 2], height: f64) {
        self.nodes[0]["children"]["modelWindow"] = json!("/cad/d1/modelWindow/1");
        self.nodes[0]["attributes"]["ifccad::drawing"]["modelWindows"] =
            json!(["/cad/d1/modelWindow/1"]);
        self.nodes[0]["attributes"]["ifccad::drawing"]["nextModelWindowId"] = json!(2);
        self.nodes[0]["attributes"]["ifccad::drawingWorkspace"] =
            json!({"activeLayout":"/cad/d1/layout/1","currentLayer":"/cad/d1/layer/0"});
        self.nodes[0]["attributes"]["ifccad::modelViewState"] =
            json!({"activeModelWindow":"/cad/d1/modelWindow/1"});
        self.nodes.push(json!({"path":"/cad/d1/modelWindow/1","attributes":{"ifccad::modelWindow":{
            "aspectRatio":1.5,"rectangle":[0.,0.,1.,1.],"renderMode":"TwoDimensional",
            "storedUcs":{"kind":"World"},"useStoredUcs":false,
            "grid":{"enabled":false,"spacing":[10.,10.],"style":"Lines","majorLineFrequency":5,"beyondLimits":false,"adaptive":false,"subdivision":false,"followsWorkplane":false},
            "snap":{"enabled":false,"base":[0.,0.],"spacing":[10.,10.],"angle":0.,"style":"Rectangular","isometricPlane":"Top"},
            "view":{"center":center,"target":[0.,0.,0.],"direction":[0.,0.,1.],"height":height,"twist":0.,"projection":"Orthographic","frontClip":{"mode":"Disabled"},"backClip":{"mode":"Disabled"}}
        }}}));
    }
    pub fn finish(mut self, id: &str) -> Result<Vec<u8>> {
        let drawing = &mut self.nodes[0]["attributes"]["ifccad::drawing"];
        drawing["nextEntityId"] = json!(self.next_entity);
        drawing["nextLayerId"] = json!(self.next_layer);
        drawing["nextBlockId"] = json!(self.next_block);
        let value = json!({"header":{"id":id,"ifcxVersion":"ifcx_alpha","dataVersion":"0.1.0","author":"OpenAEC","timestamp":"2026-10-20T00:00:00Z"},"imports":[{"uri":"urn:example:ifccad:0.1.0"}],"schemas":{},"data":self.nodes});
        let loaded = ifccad::load_ifccad_bytes(&serde_json::to_vec(&value)?, Default::default())?;
        let mut document = loaded.into_document();
        ifccad::recompute_ifccad_document_bounds(&mut document)?;
        let encoded = ifccad::encode_ifccad_document(&document)?;
        super::consistency::normalize(&serde_json::from_slice(encoded.bytes())?)
    }
}

pub fn diagonal_fill(spacing: f64) -> Value {
    json!({"kind":"linePattern","name":"Diagonal","description":"45 degree material hatch","origin":[0.,0.],"rotation":0.,"scale":1.,"families":[{"angle":std::f64::consts::FRAC_PI_4,"basePoint":[0.,0.],"offset":[0.,spacing],"dashes":[]}]})
}
