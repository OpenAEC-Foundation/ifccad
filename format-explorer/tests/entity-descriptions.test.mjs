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

test('native entities give a concise meaning while collection and fields carry storage detail',async()=>{
 const fixtures=await readExamples();
 const drawing=buildModel(fixtures.find(f=>f.name==='unrepresented-packed'));
 const blocks=buildModel(fixtures.find(f=>f.name==='blocks-demo'));
 const line=description(drawing,'entity:drawing-main:1');
 const polyline=description(drawing,'entity:drawing-main:3');
 const block=description(blocks,'entity:drawing-blocks-demo:4');
 assert.match(line,/lijn.*XYZ-eindpunten/is);
 assert.match(polyline,/polylijn.*lokale XY.*closed.*placement/is);
 assert.match(block,/block instance.*gedeelde blockdefinitie.*rotatie.*schaal/is);
 for(const value of Object.values(entityDescriptions)){
  assert.ok(value.length<150);
  assert.doesNotMatch(value,/Stream|vertexOffset|definitionScopeId|x1\/y1/i);
 }
 for(const html of [line,polyline,block])assert.doesNotMatch(html,/Een native entiteit heeft een identiteit/);
});

test('native collections explain their own stream instead of the generic collection',async()=>{
 const fixtures=await readExamples();
 const drawing=buildModel(fixtures.find(f=>f.name==='unrepresented-packed'));
 const blocks=buildModel(fixtures.find(f=>f.name==='blocks-demo'));
 assert.match(description(drawing,'collection:drawing-main:line'),/Lijnen.*lineStream.*XYZ-eindpunten/is);
 assert.match(description(drawing,'collection:drawing-main:polyline'),/Polylijnen.*polylineStream.*vertexOffset.*vertexCount/is);
 assert.match(description(blocks,'collection:drawing-blocks-demo:blockInstance'),/block instances.*blockInstanceStream.*definitionScopeId/is);
 for(const [model,id]of [[drawing,'collection:drawing-main:line'],[drawing,'collection:drawing-main:polyline'],[blocks,'collection:drawing-blocks-demo:blockInstance']]){
  assert.doesNotMatch(description(model,id),/Gelijksoortige entiteiten delen een typed stream/);
 }
});

test('native field nodes describe meaning and stored columns',async()=>{
 const fixtures=await readExamples();
 const drawing=buildModel(fixtures.find(f=>f.name==='unrepresented-packed'));
 const blocks=buildModel(fixtures.find(f=>f.name==='blocks-demo'));
 assert.match(description(drawing,'field:entity:drawing-main:1:start'),/beginpunt.*x1.*y1.*z1.*lineStream/is);
 assert.match(description(drawing,'field:entity:drawing-main:1:end'),/eindpunt.*x2.*y2.*z2.*lineStream/is);
 assert.match(description(drawing,'field:entity:drawing-main:3:vertices'),/lokale XY.*vertexOffset.*vertexCount.*x.*y/is);
 assert.match(description(drawing,'field:entity:drawing-main:3:closed'),/laatste punt.*eerste.*closed.*polylineStream/is);
 assert.match(description(drawing,'field:entity:drawing-main:3:placement'),/lokale XY.*XYZ.*placement.*polylineStream/is);
 assert.match(description(drawing,'field:entity:drawing-main:3:vertices.0'),/lokaal XY.*puntenpools/is);
 assert.match(description(blocks,'field:entity:drawing-blocks-demo:4:definitionScopeId'),/gedeelde blockdefinitie.*definitionScopeId.*blockInstanceStream/is);
 assert.match(description(blocks,'field:entity:drawing-blocks-demo:4:transform'),/plaatsing.*rotatie.*schaal.*transform/is);
 assert.match(description(blocks,'field:entity:drawing-blocks-demo:5:transform.rotation'),/rotatie.*radialen.*transform.rotation/is);
 assert.match(description(blocks,'field:entity:drawing-blocks-demo:6:transform.scale'),/schaal.*transform.scale/is);
});

test('every registered native object stream has a translated explanation',async()=>{
 const registry=JSON.parse(await readFile(new URL('../../schemas/ifcdr/registry-0.9.0.json',import.meta.url),'utf8'));
 const families=registry.streams.filter(stream=>stream.role==='object').map(stream=>stream.name);
 for(const descriptions of [entityDescriptions,collectionDescriptions,fieldDescriptions])assert.deepEqual(Object.keys(descriptions).sort(),families.sort());
 for(const value of [...Object.values(entityDescriptions),...Object.values(collectionDescriptions),...Object.values(fieldDescriptions).flatMap(fields=>Object.values(fields))]){
  assert.notEqual(t(value,'en'),value);
 }
 const fixtures=await readExamples();
 for(const fixture of fixtures){
  const model=buildModel(fixture);
  for(const entity of model.entities)for(const field of Object.keys(entity.geometry))assert.ok(fieldDescriptions[entity.kind]?.[field],`${entity.kind}.${field} needs an explanation`);
 }
});
