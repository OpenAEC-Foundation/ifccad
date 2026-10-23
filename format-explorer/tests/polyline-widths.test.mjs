import test from 'node:test';import assert from 'node:assert/strict';import {widthPresentation,effectiveWidthPresentation} from '../src/polyline-widths.mjs';import {hatchExportState} from '../src/hatch-export.mjs';
test('inherited explicit zero equal and effective labels',()=>{for(const language of ['nl','en']){const html=widthPresentation({kind:'defaulted',defaultStartWidth:.2,defaultEndWidth:.4,overrides:[{}, {start:0},{start:.2,end:.4}]},language);assert.match(html,language==='nl'?/Overgeërfd/:/Inherited/);assert.match(html,/0/);assert.doesNotMatch(html,/Effective|Effectieve/);assert.match(effectiveWidthPresentation({kind:'defaulted',defaultStartWidth:.2,defaultEndWidth:.4,overrides:[{start:0},{}]},language),language==='nl'?/Effectieve/:/Effective/);assert.ok(!widthPresentation({kind:'<script>'},language).includes('<script>'));}});
test('mixed_hatch_upright_width_preflight_preserves_policy_effects',()=>{const entry=restrictions=>[{format:'dwg',version:'AC1032',restrictions}];const p={hatchExportPreflight:entry([{policyEffect:'check'}]),linePatternExportPreflight:entry([{policyEffect:'loss',code:'UPRIGHT'}]),polylineWidthExportPreflight:entry([{policyEffect:'loss',code:'polyline-width-dependency'}])};assert.equal(hatchExportState(p,{format:'dwg',version:'AC1032',lossPolicy:'reject'}).blocked,true);assert.equal(hatchExportState(p,{format:'dwg',version:'AC1032'}).restrictions.length,3);assert.equal(hatchExportState({hatchExportPreflight:p.hatchExportPreflight},{format:'dwg',version:'AC1032',lossPolicy:'reject'}).blocked,false);});

import {createInspection} from '../src/inspection-model.mjs';
import {renderInspection} from '../src/inspection-view.mjs';
test('graphics width facts retain ownership and untrusted text is escaped',()=>{
 const drawing='/cad/d9007199254740993',dimension=drawing+'/e9007199254740995',part=dimension+'/graphics/p0';
 const widths={kind:'defaulted',defaultStartWidth:.2,defaultEndWidth:.4,overrides:[{start:'<img src=x onerror=alert(1)>'},{}]};
 const data=[{path:drawing,attributes:{'ifccad::drawing':{lengthUnit:'mm'}}},{path:dimension,attributes:{'ifccad::entity':{},'ifccad::dimension':{}},children:{graphics:dimension+'/graphics'}},{path:dimension+'/graphics',attributes:{'ifccad::dimensionGraphics':{}},children:{0:part}},{path:part,attributes:{'ifccad::dimensionGraphicsPart':{},'ifccad::geom::planarPolyline':{vertices:[[0,0],[1,0]],widths}}}];
 const result={validation:{strictAvailable:true},presentation:{graph:{data},polylineWidths:[{path:part,widths}]}};const model=createInspection(result);assert.equal(model.nodes.get(part).owner,dimension);assert.equal(model.nodes.get(part).polylineWidths,widths);
 const view=renderInspection(model,{language:'en',selection:part,inspector:'properties',expanded:new Set(),fieldExpanded:new Set(),fieldCollapsed:new Set()});assert.ok(view.details.includes('&lt;img'));assert.ok(!view.details.includes('<img src=x'));assert.match(view.details,/Inherited/);
});


test('stored polyline widths belong to geometry without conversion assessments',()=>{
 const path='/cad/d7/e2',widths={kind:'constant',width:.15};
 const model=createInspection({validation:{strictAvailable:true},presentation:{graph:{data:[{path,attributes:{'ifccad::entity':{},'ifccad::geom::planarPolyline':{vertices:[[-.5,-.5],[.5,.5]],widths}}}]},polylineWidths:[{path,widths}]}});
 for(const language of ['en','nl']){
  const html=renderInspection(model,{language,selection:path,inspector:'properties',expanded:new Set(),fieldExpanded:new Set(),fieldCollapsed:new Set()}).details;
  const stored=html.indexOf(language==='en'?'Stored values':'Opgeslagen waarden'),width=html.indexOf(language==='en'?'Constant full width':'Constante volle breedte');
  assert.ok(stored>=0&&width>stored,'authored width is under stored values');
  assert.match(html,/0\.15/);
  assert.doesNotMatch(html,/Segment band position|Joins and surface|Host display|Positie van segmentband|Aansluitingen en oppervlak|Weergave door de host/);
  assert.doesNotMatch(html,/OCDraw/);
 }
});
