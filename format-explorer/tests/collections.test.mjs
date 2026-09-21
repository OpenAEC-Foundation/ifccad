import test from 'node:test';
import assert from 'node:assert/strict';
import {createCollections} from '../src/collections.mjs';
test('large collections browse bounded windows and reveal distant targets',()=>{
 const byId=new Map([['root',{id:'root',domain:'ifcdr',x:0,y:0}]]),edges=[];
 const c=createCollections({byId,add:n=>byId.set(n.id,n),edge:(...e)=>edges.push(e),defaultCollapsed:new Set()});
 c.register('root',Array.from({length:10000},(_,i)=>i),(n)=>byId.set('e'+n,{id:'e'+n}),n=>'e'+n);
 assert.equal(byId.size,1);c.materialize('e9999');assert.ok(byId.has('e9999'));assert.ok(!byId.has('e500'));
 assert.equal([...byId.keys()].filter(id=>id.startsWith('e')&&c.isVisible(id)).length,10);
 c.setPage('root',0);assert.ok(c.isVisible('e0'));assert.ok(!c.isVisible('e9999'));
 for(let page=1;page<20;page++)c.setPage('root',page);
 assert.equal([...byId.keys()].filter(id=>id.startsWith('e')&&c.isVisible(id)).length,10);
 c.materialize('e9999');assert.ok(c.isVisible('e9999'));
 assert.equal(c.collections.get('root').start,9990);
});
