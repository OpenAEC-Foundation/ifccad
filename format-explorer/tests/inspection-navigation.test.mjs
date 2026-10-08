import test from 'node:test';import assert from 'node:assert/strict';
import {createInspection} from '../src/inspection-model.mjs';
import * as nav from '../src/inspection-navigation.mjs';
test('search and layer filters keep original identities and source order',()=>{
 assert.equal(typeof nav.collectionEntries,'function');assert.equal(typeof nav.filterCollection,'function');
 const model=createInspection({presentation:{format:'ocdraw',layers:[{id:0,name:'Ground'},{id:1,name:'Roof'}],entities:[{id:'9007199254740993',layerId:1,geometry:{type:'line',start:[1,2,0],end:[3,4,0]}},{id:2,layerId:0,geometry:{type:'circle',radius:5}},{id:3,layerId:1,geometry:{type:'line'}}]}});
 const entries=nav.collectionEntries(model,['entity:9007199254740993','entity:2','entity:3'],n=>n.type+' '+n.title);
 assert.deepEqual(nav.filterCollection(entries,{layer:'layer:1'}).map(e=>e.key),['entity:9007199254740993','entity:3']);
 assert.deepEqual(nav.filterCollection(entries,{query:'ground'}).map(e=>e.key),['entity:2']);
 assert.equal(nav.filterCollection(entries,{query:'9007199254740993',layer:'layer:1'})[0].key,'entity:9007199254740993');
 assert.equal(nav.filterCollection(entries,{query:'ground',layer:'layer:1'}).length,0);
 assert.match(entries[0].summary,/1, 2/);assert.equal(model.nodes.get('entity:9007199254740993').values.id,'9007199254740993');
});
test('virtual scroll window reaches the final item without growing with the collection',()=>{
 const window=nav.listWindow(10000,9999*56,400);assert.equal(window.end,10000);assert.ok(window.end-window.start<20);assert.equal(window.bottom,0);assert.equal(window.top,9995*56);
 assert.deepEqual(nav.listWindow(0),{start:0,end:0,top:0,bottom:0});
});
test('storage roots and draw-order entities use collections rather than unbounded tree siblings',()=>{
 const entities=Array.from({length:400},(_,i)=>({id:i+1,geometry:{type:'line'}}));
 const model=createInspection({presentation:{format:'ocdraw',entities,layouts:[{id:0,scopeId:0,name:'Model'}],scopes:[{id:0,entities:entities.map(e=>e.id)}]}});
 const order=nav.inspectionNavigation(model,{view:'drawing',grouping:'order'});assert.equal(order.forSelection('entity:400').key,'layout:0');
 const storage=nav.inspectionNavigation(model,{view:'nodes'});assert.deepEqual(storage.roots,['inspection:roots']);assert.equal(storage.forSelection('entity:400').keys.length,400);
});
test('an open middle list follows small groups and singleton types without showing unrelated records',()=>{
 const model=createInspection({presentation:{format:'ocdraw',entities:[{id:1,geometry:{type:'line'}},{id:2,geometry:{type:'circle'}},{id:3,geometry:{type:'circle'}}],layers:[{id:0,name:'Layer 0'}],layouts:[{id:0,scopeId:0,name:'Model'}],scopes:[{id:0,entities:[1,2,3]}]}});
 const navigation=nav.inspectionNavigation(model);assert.equal(typeof navigation.activeCollection,'function');
 assert.equal(navigation.activeCollection('entity:1'),null);
 assert.deepEqual(navigation.activeCollection('entity:1',true).keys,['entity:1']);
 assert.deepEqual(navigation.activeCollection('group:types:layout:0:circle',true).keys,['entity:2','entity:3']);
 assert.deepEqual(navigation.activeCollection('group:layers',true).keys,['layer:0']);
 assert.equal(navigation.activeCollection('drawing',true),null);
});
test('many distinct entity types do not hide the structural type overview behind a list',()=>{
 const entities=Array.from({length:12},(_,i)=>({id:i+1,geometry:{type:'kind'+i}}));
 const model=createInspection({presentation:{format:'ocdraw',entities,layouts:[{id:0,scopeId:0,name:'Model'}],scopes:[{id:0,entities:entities.map(e=>e.id)}]}});
 assert.equal(nav.inspectionNavigation(model).isLarge('layout:0'),false);
 assert.equal(nav.inspectionNavigation(model,{grouping:'order'}).isLarge('layout:0'),true);
});
test('native CAD navigation distinguishes layouts, entities and non-layout resources in both formats',()=>{
 assert.equal(typeof nav.cadSelectionForNode,'function');
 const ocdraw=createInspection({presentation:{format:'ocdraw',layouts:[{id:0,scopeId:0,kind:'model',name:'Model'},{id:1,scopeId:1,kind:'paper',name:'Sheet A'}],entities:[{id:2,geometry:{type:'line'}}],layers:[{id:0,name:'Layer 0'}]}});
 assert.deepEqual(nav.cadSelectionForNode(ocdraw,'layout:0'),{kind:'layout',layout:'Model'});
 assert.deepEqual(nav.cadSelectionForNode(ocdraw,'layout:1'),{kind:'layout',layout:'Sheet A'});
 assert.deepEqual(nav.cadSelectionForNode(ocdraw,'entity:2'),{kind:'element',key:'entity:2'});
 assert.equal(nav.cadSelectionForNode(ocdraw,'layer:0'),null);
 const ifccad=createInspection({presentation:{format:'ifccad',graph:{data:[{path:'/layout/m',attributes:{'ifccad::layout':{kind:'Model'}}},{path:'/layout/9007199254740993',attributes:{'ifccad::layout':{kind:'Paper',name:'Sheet B'}}},{path:'/e2',attributes:{'ifccad::entity':{},'ifccad::geom::lineSegment':{}}}]}}});
 assert.deepEqual(nav.cadSelectionForNode(ifccad,'/layout/m'),{kind:'layout',layout:'Model'});
 assert.deepEqual(nav.cadSelectionForNode(ifccad,'/layout/9007199254740993'),{kind:'layout',layout:'Sheet B'});
 assert.deepEqual(nav.cadSelectionForNode(ifccad,'/e2'),{kind:'element',key:'/e2'});
});
