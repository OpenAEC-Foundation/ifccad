import test from 'node:test';
import assert from 'node:assert/strict';
import * as formats from '../src/cad-formats.mjs';
test('roundtrip settings follow a supported source CAD format and version',()=>{
 assert.equal(typeof formats.cadOutputForSource,'function');
 assert.deepEqual(formats.cadOutputForSource('drawing.DWG','AC1027',{format:'dxf',version:'AC1032'}),{format:'dwg',version:'AC1027'});
 assert.deepEqual(formats.cadOutputForSource('drawing.dxf','AC1018',{format:'dwg',version:'AC1027'}),{format:'dxf',version:'AC1018'});
});
test('native drawings retain explicit CAD output settings and unsupported CAD versions retain a supported fallback',()=>{
 assert.equal(typeof formats.cadOutputForSource,'function');
 assert.deepEqual(formats.cadOutputForSource('drawing.ifcx',undefined,{format:'dwg',version:'AC1027'}),{format:'dwg',version:'AC1027'});
 assert.deepEqual(formats.cadOutputForSource('old.dxf','AC1009',{format:'dwg',version:'AC1018'}),{format:'dxf',version:'AC1018'});
});
