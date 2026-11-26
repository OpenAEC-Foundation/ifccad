import test from 'node:test';
import assert from 'node:assert/strict';
import {renderExternalSourceChoices} from '../src/inspection-view.mjs';
import {createDocumentSession} from '../src/ocs-messages.mjs';

test('nested choices carry owner, source and locator without filename identity guesses',()=>{
 const html=renderExternalSourceChoices({id:'1',nestedSources:[{rootSourceId:'1',name:'<detail>',ownerResource:'7',sourceId:'1',locator:'../detail/C.ifccad',status:'Circular',diagnostic:'loop'}],resources:[{id:'1',name:'Current drawing'},{id:'2',name:'same.ifccad'},{id:'3',name:'same.ifccad'}]},'nl');
 assert.match(html,/data-owner-resource="7"/);
 assert.match(html,/data-source-id="1"/);
 assert.match(html,/data-source-locator="\.\.\/detail\/C.ifccad"/);
 assert.match(html,/Cyclische verwijzing onderbroken/);
 assert.match(html,/&lt;detail&gt;/);
 assert.match(html,/value="2"/);assert.match(html,/value="3"/);
 assert.match(html,/Huidige tekening/);
});

test('reusing a resource sends its opaque identity in the generated-document scope',async()=>{
 let current=1;const docs=[{id:1}],calls=[];
 const session=createDocumentSession(async request=>{
  if(request.op==='state')return {ok:true,document_id:current,documents:docs};
  if(request.op==='open'){current=2;docs.push({id:2});return {ok:true};}
  if(request.op==='xref_source_register'){calls.push(request);return {ok:true,result:{source_status:'loaded'}};}
  throw Error(request.op);
 });
 await session.replaceGenerated('e30=','main.ifcx');
 await session.registerExternalSource('1',undefined,undefined,session.ids().generatedId,{ownerResource:'7',reuseResource:'1',locator:'../C.ifccad'});
 assert.deepEqual(calls[0],{op:'xref_source_register',document_id:2,source_id:'1',owner_resource:'7',reuse_resource:'1',locator:'../C.ifccad'});
});
