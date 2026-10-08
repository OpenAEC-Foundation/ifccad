import test from 'node:test';import assert from 'node:assert/strict';
const reports=await import('../src/inspection-report.mjs').catch(()=>({}));
test('input conversion evidence survives a successful roundtrip with distinct output restoration and readback',()=>{
 assert.equal(typeof reports.inspectionReport,'function');
 const input={reader:{status:'ok'},validation:{strictAvailable:true},conversion:{diagnostics:['input loss']},failure:{message:'previous output failed'}};
 const preview={conversion:{diagnostics:['input loss'],restoration:{preservation:{entries:['restored']}}},export:{format:'dxf',requestedVersion:'AC1032',options:{tolerance:{mode:'exact'}},geometryAssessment:{domains:['model']},diagnostics:['output loss'],fileCheck:{cadReadback:true},download:{base64:'must not be in the report'}},failure:null};
 const report=reports.inspectionReport(input,preview);
 assert.deepEqual(report.input.reader,{status:'ok'});assert.deepEqual(report.input.validation,{strictAvailable:true});
 assert.deepEqual(report.input.conversion,{diagnostics:['input loss']});assert.deepEqual(report.input.failure,{message:'previous output failed'});
 assert.deepEqual(report.output.restoration,{preservation:{entries:['restored']}});
 assert.deepEqual(report.output.export.diagnostics,['output loss']);assert.deepEqual(report.output.export.fileCheck,{cadReadback:true});
 assert.deepEqual(report.output.export.geometryAssessment,{domains:['model']});assert.equal(report.output.failure,null);
 assert.equal(report.output.reader,undefined);assert.equal(report.output.validation,undefined);assert.equal(report.output.conversion,undefined);
 assert.equal(report.output.export.download,undefined);assert.equal(preview.export.download.base64,'must not be in the report');
});

test('an output failure cannot replace input validity and failures stay in their own stage',()=>{
 const report=reports.inspectionReport({validation:{strictAvailable:true},failure:null},{failure:{message:'DWG write failed'}});
 assert.equal(report.input.validation.strictAvailable,true);assert.equal(report.input.failure,null);
 assert.deepEqual(report.output.failure,{message:'DWG write failed'});
 const blocked=reports.inspectionReport({failure:{message:'Cannot read original DWG'}});
 assert.deepEqual(blocked.input.failure,{message:'Cannot read original DWG'});assert.equal(blocked.output.failure,undefined);assert.equal(blocked.output.export,undefined);
});

test('native export conversion belongs to the output stage and successful export does not remove an input error',()=>{
 const report=reports.inspectionReport({failure:{message:'input error'}},{conversion:{preservation:{entries:['native source restored']}},export:{format:'dwg'},failure:null});
 assert.deepEqual(report.input.failure,{message:'input error'});
 assert.deepEqual(report.output.restoration,{preservation:{entries:['native source restored']}});
});

test('OCDraw basic CAD readback is visible when successful download has no detailed native file check',()=>{
 const output={failure:null,export:{format:'dwg',requestedVersion:'AC1027',fileCheck:null,download:{base64:'AQI='}}};
 assert.deepEqual(reports.inspectionReport({},output).output.readback,{cadReadback:true,versionMatched:true,version:'AC1027'});
 assert.equal(reports.inspectionReport({},{...output,failure:{message:'write failed'}}).output.readback,undefined);
 assert.equal(reports.inspectionReport({},{failure:null,export:{format:'dwg',fileCheck:null}}).output.readback,undefined);
 assert.equal(reports.inspectionReport({},{failure:null,export:{format:'ocdraw',download:{base64:'AQI='}}}).output.readback,undefined);
});

test('conversion directions follow actual input, native route and prepared output rather than later settings',()=>{
 assert.equal(typeof reports.conversionDirections,'function');
 assert.deepEqual(reports.conversionDirections({source:{name:'drawing.DWG'},inspectedFormat:'ocdraw'},{export:{format:'dwg',requestedVersion:'AC1027'}},{format:'dxf',version:'AC1032'}),{input:'DWG → OCDraw',output:'OCDraw → DWG',version:'AC1027',inputConverted:true});
 assert.deepEqual(reports.conversionDirections({source:{name:'drawing.ifcx'},inspectedFormat:'ifccad'},undefined,{format:'dxf',version:'AC1032'}),{input:'IFCCAD',output:'IFCCAD → DXF',version:'AC1032',inputConverted:false});
 assert.deepEqual(reports.conversionDirections({source:{name:'drawing.ocdraw.json'},inspectedFormat:'ocdraw'},undefined,{format:'dwg'}),{input:'OCDraw',output:'OCDraw → DWG',version:undefined,inputConverted:false});
 assert.deepEqual(reports.conversionDirections(),{input:'',output:'',version:undefined,inputConverted:false});
 assert.deepEqual(reports.conversionDirections(null,null,null),{input:'',output:'',version:undefined,inputConverted:false});
});
