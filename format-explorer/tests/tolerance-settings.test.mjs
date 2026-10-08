import test from 'node:test';import assert from 'node:assert/strict';
const module=await import('../src/tolerance-settings.mjs').catch(()=>({}));
test('tolerance policies can be applied together or to just one conversion boundary',()=>{
 assert.equal(typeof module.createTolerancePolicies,'function');const policies=module.createTolerancePolicies();
 policies.apply('both',{mode:'exact'});assert.deepEqual(policies.selected('input'),{mode:'exact'});assert.deepEqual(policies.selected('output'),{mode:'exact'});
 policies.apply('input',{mode:'custom',value:0.001,unit:'mm',coordinateFallback:1e-9});assert.equal(policies.selected('input').value,0.001);assert.deepEqual(policies.selected('output'),{mode:'exact'});
 policies.apply('output',{mode:'default'});assert.equal(policies.selected('input').value,0.001);assert.deepEqual(policies.selected('output'),{mode:'default'});
 assert.throws(()=>policies.apply('unknown',{mode:'default'}));assert.throws(()=>policies.apply('both',{mode:'custom',value:-1,unit:'drawing'}));assert.equal(policies.selected('input').value,0.001);
});
test('effective tolerance rows retain actual domain limits, scale and fallback provenance',()=>{
 assert.equal(typeof module.effectiveToleranceRows,'function');
 const input={presentation:{unit:'mm',layouts:[{id:7,scopeId:70,kind:'paper',name:'A'},{id:9,scopeId:90,kind:'paper',name:'B'}]}};
 const output={export:{options:{tolerance:{mode:'custom',unit:'mm',value:0.001,coordinateFallback:1e-9}},geometry:{domains:[
 {domain:'Drawing',coordinateMeaning:{kind:'DrawingUnit',unit:'mm'},resolvedTolerance:{lower:0.001,upper:0.001}},
 {domain:'PaperLayout(70)',coordinateMeaning:{kind:'PaperCoordinates',physicalOutputFactor:{numerator:'1',denominator:'500'}},resolvedTolerance:{lower:0.0005,upper:0.0005}},
 {domain:'PaperLayout(90)',coordinateMeaning:{kind:'PaperCoordinates',physicalOutputFactor:null},resolvedTolerance:{lower:1e-9,upper:1e-9}},
 ]}}};
 const rows=module.effectiveToleranceRows(input,output);assert.deepEqual(rows.map(row=>[row.name,row.upper,row.fallback]),[['Model',0.001,false],['A',0.0005,false],['B',1e-9,true]]);assert.equal(rows[1].physicalOutputFactor.numerator,'1');
 assert.deepEqual(module.effectiveToleranceRows(undefined,undefined),[]);
});
test('IFCCAD uint64 layout identities and unknown Model scale are kept distinct',()=>{
 const input={presentation:{layouts:[{path:'/cad/d1/layout/9007199254740993',attributes:{'ifccad::layout':{kind:'Paper',name:'Sheet'}}}]}};
 const output={export:{options:{tolerance:{mode:'custom',unit:'m',coordinateFallback:1e-9}},geometryAssessment:{domains:[{domain:{kind:'Drawing'},coordinateMeaning:{kind:'DrawingUnit',unit:'unitless'},resolvedTolerance:{lower:1e-9,upper:1e-9}},{domain:{kind:'PaperLayout',layoutId:'9007199254740993'},coordinateMeaning:{kind:'PaperCoordinates',physicalOutputFactor:null},resolvedTolerance:{lower:1e-9,upper:1e-9}}]}}};
 const rows=module.effectiveToleranceRows(input,output);assert.equal(rows[1].name,'Sheet');assert.equal(rows[0].fallback,true);assert.equal(rows[1].id,'9007199254740993');
});
