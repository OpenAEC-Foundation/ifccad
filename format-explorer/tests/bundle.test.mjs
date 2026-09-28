import test from 'node:test';
import assert from 'node:assert/strict';
import {parsePresentationJson,decodeBundle} from '../src/bundle.mjs';
test('large integer identities stay distinct while strings remain exact',()=>{
 const v=parsePresentationJson('{"entityId":[9007199254740992,9007199254740993],"s":"a\\\"9007199254740993","x":1.25e2}');
 assert.deepEqual(v.entityId,['9007199254740992','9007199254740993']);assert.equal(v.s,'a"9007199254740993');assert.equal(v.x,125);
 assert.throws(()=>parsePresentationJson('{invalid}'));
});
test('a large embedded drawing document parses without overflowing the stack',()=>{
 const document='x'.repeat(12_000_000);
 const value=parsePresentationJson(`{"document":${JSON.stringify(document)},"entityId":9007199254740993}`);
 assert.equal(value.document.length,document.length);
 assert.equal(value.entityId,'9007199254740993');
});
test('presentation keeps original documents and bounded blob previews',()=>{
 const text='{"data":[], "header":{}}';const f=decodeBundle({name:'test',documents:[{path:'package.ifcx.json',text}],blobs:[{path:'b',previewBase64:'AQI=',byteLength:9000}]});
 assert.equal(f.rawDocuments['package.ifcx.json'],text);assert.deepEqual(f.blobs.b,[1,2]);assert.equal(f.blobLengths.b,9000);
});
