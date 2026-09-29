import test from 'node:test';
import assert from 'node:assert/strict';
import {familyLayout,branchRoute} from '../src/family-layout.mjs';
import * as layout from '../src/family-layout.mjs';

const node=(id,kind)=>({id,kind,width:260,height:90,x:0,y:0});
test('Drawing branches are siblings and their item positions ignore neighbouring expansion',()=>{
 const boxes=new Map([
  node('drawing','Drawing'),node('layouts','group'),node('layers','group'),node('appearances','group'),node('representation','DrawingRepresentation'),
  node('layout-0','DrawingLayout'),node('layer-0','Layer'),node('layer-1','Layer'),node('appearance-0','Appearance'),
 ].map(n=>[n.id,n]));
 const edges=[['drawing','layouts'],['drawing','layers'],['drawing','appearances'],['drawing','representation'],['layouts','layout-0'],['layers','layer-0'],['layers','layer-1'],['appearances','appearance-0']].map(([source,target])=>({source,target,structural:true}));
 familyLayout(boxes,edges,['drawing']);
 assert.equal(new Set(['layouts','layers','appearances','representation'].map(id=>boxes.get(id).y)).size,1);
 assert.ok(boxes.get('layouts').x<boxes.get('layers').x&&boxes.get('layers').x<boxes.get('appearances').x);
 assert.equal(boxes.get('layer-0').x,boxes.get('layer-1').x);
 const before={x:boxes.get('layer-0').x,y:boxes.get('layer-0').y};
 boxes.delete('layout-0');familyLayout(boxes,edges,['drawing']);
 assert.deepEqual({x:boxes.get('layer-0').x,y:boxes.get('layer-0').y},before);
});

test('a branched structural route shares one trunk above siblings',()=>{
 const parent={x:200,y:100,width:260,height:90};
 const left={x:20,y:300,width:260,height:90},right={x:400,y:300,width:260,height:90};
 const a=branchRoute(parent,left,220),b=branchRoute(parent,right,220);
 assert.ok(a.startsWith('M330 190 L330 220'));
 assert.ok(b.startsWith('M330 190 L330 220'));
});

test('Drawing branch bus stays in the gap before display group cards',()=>{
 const parent={x:650,y:250,width:260,height:90};
 const children=[{x:200,y:380,width:260,height:90},{x:500,y:380,width:260,height:90},{x:800,y:380,width:260,height:90}];
 assert.equal(typeof layout.branchBusY,'function');
 const bus=layout.branchBusY(parent,children);
 assert.ok(bus>parent.y+parent.height);
 assert.ok(bus<children[0].y);
 for(const child of children){
  const route=branchRoute(parent,child,bus);
  assert.ok(route.includes(`L${parent.x+parent.width/2} ${bus} L${child.x+child.width/2} ${bus}`));
 }
});

test('multiple Drawing anchors reserve separate family widths',()=>{
 const boxes=new Map(['set','drawing-a','drawing-b','layers-a','layers-b'].map((id,i)=>[id,node(id,i===0?'DrawingSet':i<3?'Drawing':'group')]));
 const edges=[['set','drawing-a'],['set','drawing-b'],['drawing-a','layers-a'],['drawing-b','layers-b']].map(([source,target])=>({source,target,structural:true}));
 familyLayout(boxes,edges,['set']);
 assert.equal(boxes.get('drawing-b').x-boxes.get('drawing-a').x,1200);
 assert.ok(boxes.get('layers-b').x-boxes.get('layers-a').x>=1100);
});

test('package workspace state sits beside DrawingSet without colliding with preservation',()=>{
 const boxes=new Map([
  node('set','DrawingSet'),node('drawing','Drawing'),node('preservation','PreservationRepresentation'),node('workspace','PackageWorkspaceState'),
 ].map(n=>[n.id,n]));
 const edges=[['set','drawing'],['set','preservation']].map(([source,target])=>({source,target,structural:true}));
 familyLayout(boxes,edges,['set','workspace']);
 const set=boxes.get('set'),workspace=boxes.get('workspace'),preservation=boxes.get('preservation');
 assert.equal(workspace.y,set.y);
 assert.ok(workspace.x>set.x+set.width);
 assert.ok(workspace.x+workspace.width<preservation.x);
});

test('IFCDR display groups start at or to the right of their resource',()=>{
 const boxes=new Map(['resource','scopes','definitions','streams','workspace'].map((id,i)=>[id,node(id,i===0?'drawing-resource':'group')]));
 const edges=['scopes','definitions','streams','workspace'].map(target=>({source:'resource',target,structural:true}));
 familyLayout(boxes,edges,['resource']);
 const resource=boxes.get('resource');
 for(const id of ['scopes','definitions','streams','workspace'])assert.ok(boxes.get(id).x>=resource.x);
 assert.deepEqual(['scopes','definitions','streams','workspace'].map(id=>boxes.get(id).x),[200,500,800,1100]);
});
