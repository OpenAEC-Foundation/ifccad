import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { readExamples, exampleRoot } from '../scripts/fixtures.mjs';
import { buildModel } from '../src/model.mjs';

test('build input contains exact source documents and verified blob bytes',async()=>{
  const fixtures=await readExamples();assert.equal(fixtures.length,5);
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
