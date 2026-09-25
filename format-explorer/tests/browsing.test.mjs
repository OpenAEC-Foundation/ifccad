import test from 'node:test';
import assert from 'node:assert/strict';
import { columnTable } from '../src/browsing.mjs';
import { createCollections } from '../src/collections.mjs';

function modelFor(node){
 const byId=new Map([[node.id,node]]),paging=createCollections({byId,add:n=>byId.set(n.id,n),edge:()=>{},defaultCollapsed:new Set()});
 paging.register(node.id,node.raw.entityId,id=>byId.set('entity:'+node.resourceId+':'+id,{id:'entity:'+node.resourceId+':'+id}),id=>'entity:'+node.resourceId+':'+id);
 return {byId,paging};
}

test('column pages keep identity and property indexes aligned without rendering the whole stream',()=>{
 const count=10000,raw={count,entityId:Array.from({length:count},(_,i)=>String(900000+i)),layerId:Array.from({length:count},(_,i)=>i)};
 const node={id:'lines',resourceId:'r',raw},model=modelFor(node);
 model.paging.setPage('lines',999);
 const html=columnTable(node,model);
 assert.ok(html.includes('data-select="entity:r:909999"'));
 assert.ok(html.includes('>9999</span>'));
 assert.ok(!html.includes('data-select="entity:r:900000"'));
 assert.equal((html.match(/data-select=/g)||[]).length,10);
 assert.ok(html.includes('data-page-id="lines" data-page-kind="graph"'));
 assert.ok(!html.includes('lines:columns'));
 assert.ok(html.length<40000);
 model.paging.materialize('entity:r:900027');
 const selectedPage=columnTable(node,model);
 assert.ok(selectedPage.includes('data-select="entity:r:900020"'));
 assert.ok(selectedPage.includes('data-select="entity:r:900029"'));
 assert.ok(!selectedPage.includes('data-select="entity:r:909999"'));
});

test('polyline pools have separate pages from entity columns',()=>{
 const node={id:'poly',resourceId:'r',raw:{count:1,entityId:['1'],vertexOffset:[0],vertexCount:[200],x:Array.from({length:200},(_,i)=>i),y:Array.from({length:200},(_,i)=>-i)}};
 const model=modelFor(node);model.inspectorPages=new Map([['poly:pool',19]]);
 const html=columnTable(node,model);
 assert.ok(html.includes('data-select="entity:r:1"'));
 assert.ok(html.includes('>199</span>'));assert.ok(html.includes('>-199</span>'));
 assert.ok(html.includes('data-page-id="poly:pool"'));
});

test('a three-coordinate vertex pool stays separate from object rows',()=>{
 const node={id:'future-path',resourceId:'r',raw:{count:1,entityId:['1'],vertexOffset:[0],vertexCount:[20],x:Array.from({length:20},(_,i)=>i),y:Array.from({length:20},(_,i)=>i+100),z:Array.from({length:20},(_,i)=>i+200)}};
 const html=columnTable(node,modelFor(node));
 assert.match(html,/Puntenpool.*<code>x<\/code>.*<code>y<\/code>.*<code>z<\/code>/s);
 assert.match(html,/gedeelde x\/y\/z-puntenpool/);
});

test('planar bulges share the selected vertex window',()=>{
 const node={id:'planar',resourceId:'r',raw:{count:1,entityId:['1'],vertexOffset:[0],vertexCount:[20],x:Array.from({length:20},(_,i)=>i),y:Array.from({length:20},(_,i)=>-i),bulge:Array.from({length:20},(_,i)=>i/10)}};
 const model=modelFor(node);model.inspectorPages=new Map([['planar:pool',1]]);
 const html=columnTable(node,model);
 assert.match(html,/Puntenpool.*<code>x<\/code>.*<code>y<\/code>.*<code>bulge<\/code>/s);
 assert.match(html,/data-page-id="planar:pool"/);
 assert.match(html,/1\.9/);
 assert.doesNotMatch(html,/<code>bulge<\/code>[\s\S]*title="0"/);
});
