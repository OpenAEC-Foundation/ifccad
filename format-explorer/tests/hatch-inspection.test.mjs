import test from 'node:test';import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {createInspection} from '../src/inspection-model.mjs';
import {renderInspection,itemTitle} from '../src/inspection-view.mjs';
import {collectionEntries} from '../src/inspection-navigation.mjs';
const source=await readFile(new URL('../../examples/ifccad/hello-hatch-solid.ifcx',import.meta.url),'utf8');
test('IFCCAD Hatch type, grouping and contour-source links keep native identities',()=>{
 const graph=JSON.parse(source),first=graph.data.find(n=>n.attributes?.['ifccad::hatch']);
 const second=structuredClone(first);second.path='/cad/d1/e9007199254740993';graph.data.push(second);const layout=graph.data.find(n=>n.attributes?.['ifccad::layout']?.kind==='Model');layout.children['2']=second.path;
 const model=createInspection({presentation:{format:'ifccad',graph},validation:{strictAvailable:true}},source),node=model.nodes.get(first.path);
 assert.equal(node.type,'hatch');assert.equal(itemTitle(node,'en'),'Hatch · e1');assert.equal(itemTitle(node,'nl'),'Arcering · e1');
 assert.equal(model.nodes.get('group:types:'+layout.path+':hatch').children.length,2);
 assert.ok(node.outgoing.some(edge=>edge.field==='ifccad::hatch.loops.1.source'&&edge.target==='/cad/d1/e2'));
 assert.ok(model.nodes.get('/cad/d1/e2').incoming.some(edge=>edge.source===first.path));assert.equal(model.sourceText,source);
 const view=renderInspection(model,{language:'en',view:'drawing',selection:first.path,expanded:new Set([layout.path]),inspector:'properties'});
 assert.match(view.details,/Fill.*not assessed/i);assert.match(view.details,/2 contours/);assert.match(view.details,/data-select="\/cad\/d1\/e2"/);
 const entries=collectionEntries(model,[first.path],n=>itemTitle(n,'en'));assert.match(entries[0].summary,/Solid.*2 contours/i);
});
test('OCDraw Hatch references retain uint64 strings, storage row and independent fill evidence',()=>{
 const id='9007199254740993',target='9007199254740997',geometry={type:'hatch',fill:{kind:'solid'},areaRule:'normal',joinTolerance:1e-9,fillEvaluation:'unassessed',loops:[{sourceEntityId:target,boundary:{kind:'circle',center:[0,0],radius:1}}]};
 const model=createInspection({presentation:{format:'ocdraw',entities:[{id,geometry},{id:target,geometry:{type:'circle',radius:1}}],layouts:[{id:0,scopeId:0,name:'Model',kind:'model'}],scopes:[{id:0,entities:[id,target]}],streams:{hatchStream:{entityId:[id],loops:[geometry.loops]}}},validation:{strictAvailable:true}});
 const node=model.nodes.get('entity:'+id);assert.equal(node.type,'hatch');assert.equal(node.storage.stream,'hatchStream');assert.equal(node.storage.row,0);
 assert.ok(node.outgoing.some(edge=>edge.field==='geometry.loops.0.sourceEntityId'&&edge.target==='entity:'+target));
 const view=renderInspection(model,{language:'en',view:'drawing',selection:node.key,expanded:new Set(),inspector:'properties'});assert.match(view.details,/Fill.*not assessed/i);assert.match(view.details,/data-select="entity:9007199254740997"/);
 assert.equal(node.values.geometry.loops[0].sourceEntityId,target);
});
