import test from 'node:test';
import assert from 'node:assert/strict';
import {readExamples} from '../scripts/fixtures.mjs';
import {buildModel} from '../src/model.mjs';
import {renderInspector} from '../src/inspector.mjs';

test('block demo preserves shared geometry, signed scales and inspectable defaults',async()=>{
  const f=(await readExamples()).find(f=>f.name==='blocks-demo');
  const before=JSON.stringify(f),m=buildModel(f);
  assert.equal(m.entities.filter(e=>e.kind==='blockInstance').length,4);
  assert.equal(m.entities.filter(e=>e.kind==='line').length,3);
  assert.equal(m.missing.length,0);
  assert.equal(m.byId.get('entity:drawing-blocks-demo:6').entity.geometry.transform.scale.x,-1);
  assert.equal(m.byId.get('entity:drawing-blocks-demo:5').entity.geometry.transform.rotation,Math.PI/2);
  const elements=new Map();
  const previous=globalThis.document;
  globalThis.document={getElementById:id=>{if(!elements.has(id))elements.set(id,{});return elements.get(id);}};
  try{
    renderInspector(m,'entity:drawing-blocks-demo:4',new Set());
    assert.match(elements.get('selection-structure').innerHTML,/rotation \(rad\)<\/dt><dd>0/);
    assert.match(elements.get('selection-structure').innerHTML,/scale<\/dt><dd>\{&quot;x&quot;:1/);
    assert.match(elements.get('selection-links').innerHTML,/→ definitionScopeId/);
    renderInspector(m,'block-definition:drawing-blocks-demo:1',new Set());
    assert.match(elements.get('selection-structure').innerHTML,/basePoint<\/dt><dd>\{&quot;x&quot;:2/);
    renderInspector(m,'resource:drawing-blocks-demo',new Set());
    assert.match(elements.get('selection-structure').innerHTML,/blockDefinitionTable/);
  }finally{globalThis.document=previous;}
  assert.equal(JSON.stringify(f),before);
});
