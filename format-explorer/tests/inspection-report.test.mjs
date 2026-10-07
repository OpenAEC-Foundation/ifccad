import test from 'node:test';import assert from 'node:assert/strict';
const reports=await import('../src/inspection-report.mjs').catch(()=>({}));
test('CAD preview evidence is centralized without losing native reading and validation',()=>{
 assert.equal(typeof reports.inspectionReport,'function');
 const input={reader:{status:'ok'},validation:{strictAvailable:true},conversion:{diagnostics:['input loss']},failure:{message:'previous output failed'}};
 const preview={conversion:{diagnostics:['input loss'],restoration:{preservation:{entries:['restored']}}},export:{format:'dxf',requestedVersion:'AC1032',options:{tolerance:{mode:'exact'}},geometryAssessment:{domains:['model']},diagnostics:['output loss'],fileCheck:{cadReadback:true},download:{base64:'must not be in the report'}},failure:null};
 const report=reports.inspectionReport(input,preview);
 assert.deepEqual(report.reader,input.reader);assert.deepEqual(report.validation,input.validation);
 assert.deepEqual(report.conversion,preview.conversion);assert.deepEqual(report.export.diagnostics,['output loss']);
 assert.deepEqual(report.export.geometryAssessment,preview.export.geometryAssessment);assert.equal(report.failure,null);
 assert.equal(report.export.download,undefined);assert.equal(preview.export.download.base64,'must not be in the report');
});
