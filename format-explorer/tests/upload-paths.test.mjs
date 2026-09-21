import test from 'node:test';
import assert from 'node:assert/strict';
import {normalizeUploadPath,validateUpload} from '../scripts/upload-paths.mjs';
test('selected paths are safe on Windows and POSIX',()=>{
 for(const p of ['../escape','C:/escape','/escape','a\\..\\escape','CON','con.txt','dir/file:stream','dir/name.','dir/name ','a//b','a/%2e%2e/b']) assert.throws(()=>normalizeUploadPath(p),p);
 assert.equal(normalizeUploadPath('resources/drawing.ifcdr.json'),'resources/drawing.ifcdr.json');
});
test('upload limits and aliases cannot overwrite selected files',()=>{
 const f=(path,base64='e30=')=>({path,base64});
 assert.throws(()=>validateUpload({kind:'package',files:[f('package.ifcx.json'),f('PACKAGE.IFCX.JSON')]}));
 assert.throws(()=>validateUpload({kind:'package',files:[f('package.ifcx.json','%%%')]}));
 assert.throws(()=>validateUpload({kind:'cad',files:[f('test.exe')]}));
 assert.equal(validateUpload({kind:'package',files:[f('package.ifcx.json')]}).files.length,1);
});
