import test from 'node:test';import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {createInspection} from '../src/inspection-model.mjs';
import {renderInspection,itemTitle} from '../src/inspection-view.mjs';
import {collectionEntries} from '../src/inspection-navigation.mjs';
import {hatchSummary,hatchAssociationStatus} from '../src/hatch-view.mjs';
const source=await readFile(new URL('../../tests/fixtures/ifccad/hello-hatch-solid.ifcx',import.meta.url),'utf8');
test('pattern summary identifies authored line families in both languages',()=>{
 const value={fill:{kind:'linePattern',families:[{},{}]},loops:[{},{}],areaRule:'normal'};
 assert.match(hatchSummary(value,'en'),/Pattern.*2 line families/);
 assert.match(hatchSummary(value,'nl'),/Lijnpatroon.*2 lijnfamilies/);
});

test('gradient summaries name the active shape and disclose incomplete stored settings',()=>{
 const value={fill:{kind:'gradient'},gradient:{type:'inverseSpherical',colors:[{rgb:[220,80,30]},{rgb:[30,60,150]}]},loops:[],areaRule:'normal'};
 assert.match(hatchSummary(value,'en'),/Gradient.*Inverse spherical/);
 value.fill={kind:'solid'};value.gradient={rotation:.5,tint:.65};
 assert.match(hatchSummary(value,'nl'),/Effen.*onvolledig/i);
 value.fill={kind:'unknown'};
 assert.doesNotMatch(hatchSummary(value,'en'),/Solid/);
});
test('IFCCAD Hatch type, grouping and contour-source links keep native identities',()=>{
 const graph=JSON.parse(source),first=graph.data.find(n=>n.attributes?.['ifccad::hatch']);
 const second=structuredClone(first);second.path='/cad/d1/e9007199254740993';graph.data.push(second);const layout=graph.data.find(n=>n.attributes?.['ifccad::layout']?.kind==='Model');layout.children['2']=second.path;
 const model=createInspection({presentation:{format:'ifccad',graph},validation:{strictAvailable:true}},source),node=model.nodes.get(first.path);
 assert.equal(node.type,'hatch');assert.equal(itemTitle(node,'en'),'Hatch · e1');assert.equal(itemTitle(node,'nl'),'Arcering · e1');
 assert.equal(model.nodes.get('group:types:'+layout.path+':hatch').children.length,2);
 assert.ok(node.outgoing.some(edge=>edge.field==='ifccad::hatch.loops.1.sources.0'&&edge.target==='/cad/d1/e2'));
 assert.ok(model.nodes.get('/cad/d1/e2').incoming.some(edge=>edge.source===first.path));assert.equal(model.sourceText,source);
 const view=renderInspection(model,{language:'en',view:'drawing',selection:first.path,expanded:new Set([layout.path]),inspector:'properties'});
 assert.match(view.details,/Fill.*not assessed/i);assert.match(view.details,/2 contours/);assert.match(view.details,/data-select="\/cad\/d1\/e2"/);
 const entries=collectionEntries(model,[first.path],n=>itemTitle(n,'en'));assert.match(entries[0].summary,/Solid.*2 contours/i);
});


test('Hatch association state distinguishes stored disabled groups and unbound switch',()=>{
 for(const [enabled,sources,status] of [[false,['9'],'disabled'],[true,['9'],'active'],[true,[],'unbound']]){
  const value={associationEnabled:enabled,loops:[{sources,textIsland:true}],fill:{kind:'solid'},areaRule:'normal'};
  assert.equal(hatchAssociationStatus(value),status);
  assert.match(hatchSummary(value,'en'),new RegExp(status,'i'));
  assert.equal(value.loops[0].textIsland,true);
 }
});
