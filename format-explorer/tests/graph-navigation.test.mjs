import test from 'node:test';
import assert from 'node:assert/strict';
import {createCollections} from '../src/collections.mjs';
import {expandExclusiveStream} from '../src/graph-navigation.mjs';

test('independent three-row graph windows cross internal lazy-loading chunks',()=>{
 const nodes=new Map([['a',{id:'a'}],['b',{id:'b'}]]),loaded=[];
 const paging=createCollections({add:n=>nodes.set(n.id,n),edge:()=>{},byId:nodes,defaultCollapsed:new Set(),pageSize:10});
 for(const id of ['a','b'])paging.register(id,Array.from({length:24},(_,i)=>i),(item)=>{loaded.push(id+':'+item);nodes.set(id+':'+item,{id:id+':'+item});},item=>id+':'+item);
 paging.expand('a');paging.expand('b');
 assert.equal(paging.graphWindow('a').end,3);
 paging.scrollGraph('a',9);
 assert.deepEqual([paging.graphWindow('a').start,paging.graphWindow('a').end],[9,12]);
 assert.deepEqual([paging.collections.get('a').start,paging.collections.get('a').end],[0,10]);
 assert.ok(loaded.includes('a:11'));
 assert.equal(paging.isGraphVisible('a:11'),true);
 assert.equal(paging.isGraphVisible('a:12'),false);
 assert.equal(paging.graphWindow('b').start,0);
 paging.scrollGraph('a',1);
 assert.equal(paging.collections.get('a').start,10);
 paging.materialize('a:23');
 assert.deepEqual([paging.graphWindow('a').start,paging.graphWindow('a').end],[21,24]);
 assert.equal(paging.collections.get('a').start,20);
});

test('expanding one entity stream folds only siblings in the same IFCDR resource',()=>{
 const nodes=new Map(['line','polyline','other'].map((family,i)=>[`collection:${i===2?'other':'drawing'}:${family}`,{id:`collection:${i===2?'other':'drawing'}:${family}`,kind:'collection',domain:'ifcdr',resourceId:i===2?'other':'drawing'}]));
 nodes.set('entity:drawing:1',{id:'entity:drawing:1',collectionId:'collection:drawing:line'});
 const collapsed=new Set(['collection:drawing:polyline']);
 const selected=expandExclusiveStream({byId:nodes},collapsed,'collection:drawing:polyline','entity:drawing:1');
 assert.equal(selected,'collection:drawing:polyline');
 assert.ok(collapsed.has('collection:drawing:line'));
 assert.ok(!collapsed.has('collection:drawing:polyline'));
 assert.ok(!collapsed.has('collection:other:other'));
});
