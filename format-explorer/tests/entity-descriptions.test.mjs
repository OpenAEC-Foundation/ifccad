import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {readExamples} from '../scripts/fixtures.mjs';
import {buildModel} from '../src/model.mjs';
import {collectionDescriptions,entityDescriptions,fieldDescriptions,renderInspector} from '../src/inspector.mjs';
import {t} from '../src/i18n.mjs';

function description(model,id){
 const elements=new Map(),previous=globalThis.document;
 globalThis.document={getElementById:key=>{if(!elements.has(key))elements.set(key,{});return elements.get(key);}};
 try{renderInspector(model,id,new Set());return elements.get('selection-content').innerHTML;}
 finally{globalThis.document=previous;}
}

function structure(model,id){
 const elements=new Map(),previous=globalThis.document;
 globalThis.document={getElementById:key=>{if(!elements.has(key))elements.set(key,{});return elements.get(key);}};
 try{renderInspector(model,id,new Set());return elements.get('selection-structure').innerHTML;}
 finally{globalThis.document=previous;}
}

test('paper layout and viewport explain native scope, plot and child stream data',async()=>{
 const fixture=(await readExamples()).find(f=>f.name==='layout-viewport-plot'),model=buildModel(fixture);
 assert.match(description(model,'ifcx:layout-1'),/paperspace-scope.*plotinstellingen/is);
 assert.match(structure(model,'ifcx:layout-1'),/Effectieve plotinstellingen.*media.*area.*mapping.*output.*options/is);
 assert.match(description(model,'collection:geometry:viewport'),/viewportStream.*viewScopeId.*viewportLayerOverrideStream/is);
 assert.match(description(model,'entity:geometry:1'),/viewport.*paperspace.*modelspace/is);
 assert.match(description(model,'field:entity:geometry:1:layerOverrides'),/layerOverrideOffset.*layerOverrideCount/is);
 assert.match(structure(model,'field:entity:geometry:1:layerOverrides'),/appearanceOverrideId/is);
});

test('current drawings list definitions while older packages still explain absent lists',async()=>{
 const fixtures=await readExamples();
 const currentSource=fixtures.find(f=>f.name==='unrepresented-packed');
 const legacySource=structuredClone(currentSource);
 const drawing=legacySource.ifcx.data.find(n=>n.type==='openaec:Drawing');
 delete drawing.children.Layers;
 delete drawing.children.Appearances;
 const legacy=buildModel(legacySource);
 const migrated=buildModel(currentSource);
 const current=buildModel(fixtures.find(f=>f.name==='layout-viewport-plot'));
 assert.match(structure(legacy,'ifcx:drawing-main'),/ouder pakketcontract.*Layers.*Appearances/is);
 assert.doesNotMatch(structure(migrated,'ifcx:drawing-main'),/ouder pakketcontract/is);
 assert.doesNotMatch(structure(current,'ifcx:drawing-0'),/ouder pakketcontract/is);
});

test('native entities give a concise meaning while collection and fields carry storage detail',async()=>{
 const fixtures=await readExamples();
 const drawing=buildModel(fixtures.find(f=>f.name==='unrepresented-packed'));
 const blocks=buildModel(fixtures.find(f=>f.name==='blocks-demo'));
 const line=description(drawing,'entity:drawing-main:1');
 const polyline=description(drawing,'entity:drawing-main:3');
 const block=description(blocks,'entity:drawing-blocks-demo:4');
 assert.match(line,/lijn.*XYZ-eindpunten/is);
 assert.match(polyline,/polylijn.*lokale XY.*Bulges.*placement/is);
 assert.match(block,/block instance.*gedeelde blockdefinitie.*rotatie.*schaal/is);
 for(const value of Object.values(entityDescriptions)){
  assert.ok(value.length<150);
  assert.doesNotMatch(value,/Stream|vertexOffset|definitionScopeId|x1\/y1/i);
 }
 for(const html of [line,polyline,block])assert.doesNotMatch(html,/Een native entiteit heeft een identiteit/);
});

test('a not-yet-explained object stream has honest structural descriptions',async()=>{
 const fixture=structuredClone((await readExamples()).find(f=>f.name==='unrepresented-packed'));
 const body=fixture.files['drawing.ifcdr.json'];
 body.streamDirectory.streams.push({name:'futureShape',schema:'example.futureShape.v1',role:'object',count:1,columns:['entityId','scopeId','radius','layerId','appearanceId']});
 body.streams.futureShapeStream={count:1,entityId:[100],scopeId:[0],radius:[12],layerId:[0],appearanceId:[0]};
 const model=buildModel(fixture);
 assert.match(description(model,'collection:drawing-main:futureShape'),/geregistreerde objectstream.*kolommen.*typespecifieke uitleg/is);
 assert.match(description(model,'entity:drawing-main:100'),/opgeslagen velden.*typespecifieke uitleg/is);
 assert.match(description(model,'field:entity:drawing-main:100:radius'),/opgeslagen veldwaarde.*zonder.*betekenis/is);
});

test('native collections explain their own stream instead of the generic collection',async()=>{
 const fixtures=await readExamples();
 const drawing=buildModel(fixtures.find(f=>f.name==='unrepresented-packed'));
 const blocks=buildModel(fixtures.find(f=>f.name==='blocks-demo'));
 assert.match(description(drawing,'collection:drawing-main:line'),/Lijnen.*lineStream.*XYZ-eindpunten/is);
 assert.match(description(drawing,'collection:drawing-main:planarPolyline'),/Vlakke polylijnen.*planarPolylineStream.*vertexOffset.*vertexCount/is);
 assert.match(description(blocks,'collection:drawing-blocks-demo:blockInstance'),/block instances.*blockInstanceStream.*definitionScopeId/is);
 for(const [model,id]of [[drawing,'collection:drawing-main:line'],[drawing,'collection:drawing-main:planarPolyline'],[blocks,'collection:drawing-blocks-demo:blockInstance']]){
  assert.doesNotMatch(description(model,id),/Gelijksoortige entiteiten delen een typed stream/);
 }
});

test('native field nodes describe meaning and stored columns',async()=>{
 const fixtures=await readExamples();
 const drawing=buildModel(fixtures.find(f=>f.name==='unrepresented-packed'));
 const blocks=buildModel(fixtures.find(f=>f.name==='blocks-demo'));
 assert.match(description(drawing,'field:entity:drawing-main:1:start'),/beginpunt.*x1.*y1.*z1.*lineStream/is);
 assert.match(description(drawing,'field:entity:drawing-main:1:end'),/eindpunt.*x2.*y2.*z2.*lineStream/is);
 assert.match(description(drawing,'field:entity:drawing-main:3:vertices'),/lokale XY.*x- en y-puntenpools.*vertexOffset.*vertexCount/is);
 assert.match(description(drawing,'field:entity:drawing-main:3:closed'),/closed.*laatste punt.*eerste/is);
 assert.match(description(drawing,'field:entity:drawing-main:3:placement'),/lokale XY.*XYZ.*standaard XY-vlak/is);
 assert.match(description(drawing,'field:entity:drawing-main:3:vertices.0'),/lokaal XY.*gedeelde puntenpool/is);
 assert.match(description(blocks,'field:entity:drawing-blocks-demo:4:definitionScopeId'),/gedeelde blockdefinitie.*definitionScopeId.*blockInstanceStream/is);
 assert.match(description(blocks,'field:entity:drawing-blocks-demo:4:transform'),/plaatsing.*rotatie.*schaal.*transform/is);
 assert.match(description(blocks,'field:entity:drawing-blocks-demo:5:transform.rotation'),/rotatie.*radialen.*transform.rotation/is);
 assert.match(description(blocks,'field:entity:drawing-blocks-demo:6:transform.scale'),/schaal.*transform.scale/is);
});

test('every current registered native object stream has a translated explanation',async()=>{
 const registry=JSON.parse(await readFile(new URL('../../schemas/ifcdr/registry-0.11.0.json',import.meta.url),'utf8'));
 const families=registry.streams.filter(stream=>stream.role==='object').map(stream=>stream.name);
 for(const descriptions of [entityDescriptions,collectionDescriptions,fieldDescriptions]){
  for(const family of families)assert.ok(Object.hasOwn(descriptions,family),family+' needs an explanation');
  assert.ok(Object.hasOwn(descriptions,'polyline'),'the supported 0.10 polyline needs an explanation');
 }
 for(const value of [...Object.values(entityDescriptions),...Object.values(collectionDescriptions),...Object.values(fieldDescriptions).flatMap(fields=>Object.values(fields))]){
  assert.notEqual(t(value,'en'),value);
 }
 const fixtures=await readExamples();
 for(const fixture of fixtures){
  const model=buildModel(fixture);
  for(const entity of model.entities)for(const field of Object.keys(entity.geometry))assert.ok(fieldDescriptions[entity.kind]?.[field],`${entity.kind}.${field} needs an explanation`);
 }
});
