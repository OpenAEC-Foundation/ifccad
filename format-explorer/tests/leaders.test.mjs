import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createInspection} from '../src/inspection-model.mjs';
import {createCadPreviewController} from '../src/cad-preview.mjs';
const graph=JSON.parse(readFileSync(new URL('../../conformance/next/ifccad/valid/leaders-family.ifcx',import.meta.url),'utf8'));
const owner=graph.data.find(n=>n.attributes?.['ifccad::multiLeader']).path;
const line=graph.data.find(n=>n.attributes?.['ifccad::multiLeaderLine']).path;
test('leader parts form one owned tree and retain their branch structure',()=>{
 const model=createInspection({validation:{strictAvailable:true},presentation:{graph}},JSON.stringify(graph));
 assert.equal(model.nodes.get(owner).type,'multiLeader');
 assert.equal(model.nodes.get(line).owner,owner);
 assert.ok(model.nodes.get(owner).children.includes(owner+'/leaders'));
 assert.ok(!model.nodes.get('group:other')?.children.includes(line));
});
test('an owned leader part selects its ordinary MLeader; reverse selection reveals that root',async()=>{
 const selected=[],revealed=[];let observe;
 const controller=createCadPreviewController({onCadSelection:p=>revealed.push(p),openSession:async()=>({replaceGenerated:async()=>{},selectGenerated:async(...a)=>selected.push(a),subscribeSelection(fn){observe=fn;},close(){}})});
 controller.setSource({kind:'ifccad',name:'leaders.ifcx'},true,{validation:{strictAvailable:true},nativeFile:{format:'ifccad',fileName:'leaders.ifcx',base64:'e30=',byteLength:2},preview:{viewerSelection:{format:'ifccad',entities:[{path:owner,handle:'A',layout:'Model'}],ownedPartOwners:[{path:line,ownerPath:owner,handle:'A',layout:'Model'}]}}});
 await controller.show();await controller.selectElement(line);assert.deepEqual(selected.at(-1),[['A'],'Model']);
 await controller.selectElement(null);observe({handles:['A'],layout:'Model'});assert.deepEqual(revealed,[owner]);
});
