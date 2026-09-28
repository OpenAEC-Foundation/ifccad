import test from 'node:test';
import assert from 'node:assert/strict';
import {zipBrowserFiles} from '../src/browser-zip.mjs';
import {zipFiles} from '../scripts/package-zip.mjs';

test('browser package ZIP matches the existing transport archive byte for byte',()=>{
 const files=[{path:'package.ifcx.json',bytes:new TextEncoder().encode('{"data":[]}')},{path:'resources/é.ifcdr.json',bytes:new Uint8Array([0,1,2,255])}];
 assert.deepEqual(zipBrowserFiles(files),new Uint8Array(zipFiles(files)));
});
