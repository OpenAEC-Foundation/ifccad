import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { readExamples, exampleRoot } from '../scripts/fixtures.mjs';
import { buildModel } from '../src/model.mjs';

test('build input contains exact source documents and verified blob bytes',async()=>{
  const fixtures=await readExamples();assert.equal(fixtures.length,9);
  for(const fixture of fixtures){
    const folder=exampleRoot(fixture.name);
    assert.deepEqual(fixture.ifcx,JSON.parse(await readFile(new URL('package.ifcx.json',folder),'utf8')));
    const model=buildModel(fixture);
    for(const resource of [...model.resources,...model.preservations]){
      if(resource.storage==='extern'){
        const source=await readFile(new URL(resource.descriptor.uri,folder));
        assert.equal(resource.descriptor.checksum,'sha256:'+createHash('sha256').update(source).digest('hex'));
        assert.deepEqual(resource.body,JSON.parse(source));
      }
    }
    for(const preservation of model.preservations)for(const blob of preservation.body.blobs){
      const bytes=Buffer.from(fixture.blobs[blob.uri]);
      assert.equal(bytes.length,blob.byteLength);
      assert.equal(blob.sha256,'sha256:'+createHash('sha256').update(bytes).digest('hex'));
      for(const record of preservation.body.records)if(record.payload){const p=record.payload;assert.ok(p.offset+p.length<=bytes.length);assert.equal(record.payloadDigest,'sha256:'+createHash('sha256').update(bytes.subarray(p.offset,p.offset+p.length)).digest('hex'));}
    }
  }
});

test('curated examples cover the current package structures',async()=>{
  const fixtures=await readExamples();
  assert.ok(fixtures.some(f=>f.ifcx.data.some(n=>n.type==='openaec:PreservationRepresentation')));
  assert.ok(fixtures.some(f=>f.ifcx.data.some(n=>n.attributes?.resource?.content)));
  assert.ok(fixtures.some(f=>f.ifcx.data.filter(n=>n.type==='openaec:Drawing').length>1));
  assert.ok(fixtures.some(f=>Object.values(f.files).some(b=>b.streams?.blockInstanceStream?.count>1)));
  assert.ok(fixtures.some(f=>Object.values(f.files).some(b=>b.streams?.viewportStream?.count>0)));
  assert.ok(fixtures.some(f=>Object.values(f.files).some(b=>b.streams?.planarPolylineStream?.placement?.some(Boolean))));
  for(const stream of ['point','circle','arc','ellipse','ellipseArc','planarPolyline','spatialPolyline'])assert.ok(fixtures.some(f=>Object.values(f.files).some(b=>b.streams?.[stream+'Stream']?.count>0)),stream+' lacks an example');
  assert.ok(fixtures.some(f=>Object.values(f.files).some(b=>b.streams?.planarPolylineStream?.bulge?.some(Boolean))));
  for(const fixture of fixtures){
    for(const node of fixture.ifcx.data){
      if(node.type==='openaec:Drawing'){
        assert.ok(Array.isArray(node.children.Layers),fixture.name+' lacks Drawing.Layers');
        assert.ok(Array.isArray(node.children.Appearances),fixture.name+' lacks Drawing.Appearances');
      }
      if(node.attributes?.resource?.format==='openaec.ifcdr'){
        const version=fixture.name==='blocks-demo'?'0.10.0':'0.11.0';
        assert.equal(node.attributes.resource.version,version,fixture.name);
        const body=node.attributes.resource.content??fixture.files[node.attributes.resource.uri];
        assert.equal(body.header.version,version,fixture.name);
      }
    }
  }
});
