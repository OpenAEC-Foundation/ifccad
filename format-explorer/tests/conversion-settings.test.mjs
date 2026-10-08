import test from 'node:test';import assert from 'node:assert/strict';
const settings=await import('../src/conversion-settings.mjs').catch(()=>({}));
test('custom tolerance requires an explicit unit and finite nonnegative value',()=>{
 assert.equal(typeof settings.normalizeTolerance,'function');
 for(const value of ['',-1,Infinity,NaN])assert.throws(()=>settings.normalizeTolerance({mode:'custom',value,unit:'mm'}));
 assert.throws(()=>settings.normalizeTolerance({mode:'custom',value:1,unit:'px'}));
 assert.deepEqual(settings.normalizeTolerance({mode:'exact'}),{mode:'exact'});
 assert.deepEqual(settings.normalizeTolerance({mode:'custom',value:'0.005',unit:'drawing'}),{mode:'custom',value:0.005,unit:'drawing'});
});
test('physical tolerance carries only an explicit valid coordinate fallback',()=>{
 assert.deepEqual(settings.normalizeTolerance({mode:'custom',value:'0.001',unit:'mm',coordinateFallback:'1e-9'}),{mode:'custom',value:0.001,unit:'mm',coordinateFallback:1e-9});
 for(const value of ['',-1,Infinity,NaN])assert.throws(()=>settings.normalizeTolerance({mode:'custom',value:1,unit:'mm',coordinateFallback:value}));
 assert.throws(()=>settings.normalizeTolerance({mode:'custom',value:1,unit:'drawing',coordinateFallback:1e-9}));
});
