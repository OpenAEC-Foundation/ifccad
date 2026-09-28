import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {initSync,open_cad,open_package,export_package} from '../wasm-build/ifccad_browser.js';

initSync({module:await readFile(new URL('../wasm-build/ifccad_browser_bg.wasm',import.meta.url))});
const paths=['package.ifcx.json','resources/drawing.ifcdr.json'];
const contents=await Promise.all(paths.map(path=>readFile(new URL(`../examples/blocks-demo/${path}`,import.meta.url))));
const packageResult=JSON.parse(open_package('blocks-demo',paths,contents));
assert.equal(packageResult.validation.strictAvailable,true);
assert.equal(packageResult.failure,null);

const exported=JSON.parse(export_package('blocks-demo',paths,contents,'drawing-0','dxf','AC1032'));
assert.equal(exported.failure,null);
assert.equal(exported.export.fileCheck.readable,true);
const dxf=Buffer.from(exported.export.download.base64,'base64');
const reopened=JSON.parse(open_cad('roundtrip.dxf','dxf',dxf,'2026-09-25T12:00:00Z'));
assert.equal(reopened.validation.strictAvailable,true);
assert.equal(reopened.failure,null);
console.log('Browser WASM package, DXF export, and DXF import verified');
