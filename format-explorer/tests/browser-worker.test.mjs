import test from 'node:test';import assert from 'node:assert/strict';
import {processBrowserRequest} from '../src/browser-worker.mjs';
const bytes=new TextEncoder().encode('demo').buffer;

test('custom options cannot silently fall back to an old WASM converter',()=>{
 const request={kind:'cad',drawingFormat:'ifccad',name:'a.dxf',files:[{path:'a.dxf',bytes}],conversionOptions:{tolerance:{mode:'exact'}}};
 const wasm={convert_cad_to_ifccad(){return '{"validation":{"strictAvailable":true}}';}};
 assert.throws(()=>processBrowserRequest(request,wasm),/options|tolerance/i);
});
test('coordinate fallback requires an advertised capable WASM converter',()=>{
 let invoked=false;const wasm={convert_cad_to_ifccad_with_options(){invoked=true;return '{"validation":{"strictAvailable":true}}';}};
 const request={kind:'cad',name:'a.dxf',files:[{path:'a.dxf',bytes}],conversionOptions:{tolerance:{mode:'custom',value:0.001,unit:'mm',coordinateFallback:1e-9}}};
 assert.throws(()=>processBrowserRequest(request,wasm),/fallback/i);assert.equal(invoked,false);
 wasm.conversion_capabilities=()=>'{"ifccad":{"coordinateToleranceFallback":true}}';processBrowserRequest(request,wasm);assert.equal(invoked,true);
});
test('an advertised older physical default cannot silently satisfy the new coordinate default',()=>{
 let invoked=false;const wasm={conversion_capabilities:()=>'{"ifccad":{"adjustableTolerance":true}}',convert_cad_to_ifccad_with_options(){invoked=true;return '{"validation":{"strictAvailable":true}}';}};
 const request={kind:'cad',name:'a.dxf',files:[{path:'a.dxf',bytes}],conversionOptions:{tolerance:{mode:'default'}}};
 assert.throws(()=>processBrowserRequest(request,wasm),/default tolerance/i);assert.equal(invoked,false);
 wasm.conversion_capabilities=()=>'{"ifccad":{"coordinateToleranceDefault":true}}';processBrowserRequest(request,wasm);assert.equal(invoked,true);
});
test('configured conversions pass their options and retain exact native source text',()=>{
 let selected;const options={tolerance:{mode:'custom',value:0.001,unit:'mm'}};
 const wasm={convert_cad_to_ifccad_with_options(name,format,data,time,text){selected=JSON.parse(text);return '{"validation":{"strictAvailable":true},"export":{"download":{"base64":"e30="}}}';}};
 const result=processBrowserRequest({kind:'cad',name:'a.dxf',files:[{path:'a.dxf',bytes}],conversionOptions:options},wasm);
 assert.deepEqual(selected,options);assert.equal(result.nativeSourceText,'{}');
});
test('CAD input and roundtrip output receive independent tolerance options',()=>{
 const input={tolerance:{mode:'exact'}},output={tolerance:{mode:'custom',value:1e-9,unit:'drawing'}},calls=[];
 const wasm={convert_cad_to_ifccad_with_options(name,format,data,time,options){calls.push(JSON.parse(options));return '{"validation":{"strictAvailable":true},"export":{"download":{"base64":"e30="}}}';},export_ifccad_with_options(name,data,format,version,options){calls.push(JSON.parse(options));return '{"validation":{"strictAvailable":true},"export":{"format":"dxf"}}';}};
 processBrowserRequest({kind:'cad',name:'a.dxf',files:[{path:'a.dxf',bytes}],conversionOptions:input,exportConversionOptions:output,export:{format:'dxf'}},wasm);
 assert.deepEqual(calls,[input,output]);
});
test('standalone opening uses one local file, without a package entry',()=>{let received;const wasm={open_ifccad(...args){received=args;return '{"validation":{"strictAvailable":true},"presentation":{"entityId":9007199254740993}}';}};const result=processBrowserRequest({kind:'ifccad',name:'demo',files:[{path:'demo.ifcx',bytes}]},wasm);assert.equal(received[0],'demo');assert.equal(result.presentation.entityId,'9007199254740993');});
test('CAD conversion supplies checked standalone bytes to CAD export',()=>{let received;const wasm={convert_cad_to_ifccad(){return JSON.stringify({source:{format:'dxf'},conversion:{diagnostics:['loss']},validation:{strictAvailable:true},export:{download:{base64:'e30='}}});},export_ifccad(...args){received=args;return '{"validation":{"strictAvailable":true},"export":{"format":"dwg"}}';}};const result=processBrowserRequest({kind:'cad',name:'a.dxf',files:[{path:'a.dxf',bytes}],export:{format:'dwg',version:'AC1032'}},wasm);assert.equal(new TextDecoder().decode(received[1]),'{}');assert.equal(received[2],'dwg');assert.deepEqual(result.conversion.diagnostics,['loss']);});
test('failed validation never supplies downloadable bytes',()=>{const wasm={open_ifccad(){return '{"validation":{"strictAvailable":false}}';},export_ifccad(){throw Error('must not export');}};const result=processBrowserRequest({kind:'ifccad',files:[{path:'a.ifcx',bytes}],export:{format:'dxf'}},wasm);assert.equal(result.export,undefined);});
test('retired package selections and unsafe paths are rejected',()=>{for(const request of [{kind:'package',files:[{path:'package.ifcx.json',bytes}]},{kind:'ifccad',files:[{path:'../a.ifcx',bytes}]},{kind:'ifccad',files:[{path:'a.ifcx',bytes},{path:'b.ifcx',bytes}]}])assert.throws(()=>processBrowserRequest(request,{}));});
test('IFCCAD opening and CAD export use the direct IFCCAD adapter',()=>{
 let exported;
 const wasm={open_ifccad(){return JSON.stringify({validation:{strictAvailable:true},presentation:{format:'ifccad'}});},export_ifccad(...args){exported=args;return JSON.stringify({validation:{strictAvailable:true},export:{format:'dxf'}});},open_drawing(){throw Error('OCDraw route must not run');}};
 const request={kind:'ifccad',name:'hello.ifcx',files:[{path:'hello.ifcx',bytes}]};
 assert.equal(processBrowserRequest(request,wasm).presentation.format,'ifccad');
 processBrowserRequest({...request,export:{format:'dxf',version:'AC1032'}},wasm);
 assert.equal(exported[2],'dxf');assert.deepEqual([...exported[1]],[...new Uint8Array(bytes)]);
});
test('CAD can roundtrip via IFCCAD without an OCDraw intermediate',()=>{
 let received;
 const wasm={convert_cad_to_ifccad(){return JSON.stringify({source:{format:'dwg'},conversion:{diagnostics:['ifcx-loss']},validation:{strictAvailable:true},export:{format:'ifccad',download:{base64:'e30='}}});},export_ifccad(...args){received=args;return JSON.stringify({validation:{strictAvailable:true},export:{format:'dxf'}});},convert_cad_to_drawing(){throw Error('wrong converter');}};
 const request={kind:'cad',drawingFormat:'ifccad',name:'a.dwg',files:[{path:'a.dwg',bytes}]};
 assert.equal(processBrowserRequest({...request,export:{format:'ifccad'}},wasm).export.format,'ifccad');
 const result=processBrowserRequest({...request,export:{format:'dxf'}},wasm);
 assert.equal(new TextDecoder().decode(received[1]),'{}');assert.deepEqual(result.conversion.diagnostics,['ifcx-loss']);
});

test('IFCCAD spline capture forwards the option and requires an advertised capable processor',()=>{
 let selected;
 const wasm={conversion_capabilities(){return JSON.stringify({ifccad:{adjustableTolerance:true,coordinateToleranceDefault:true,splinePreservation:true}});},convert_cad_to_ifccad_with_options(name,format,input,time,text){selected=JSON.parse(text);return JSON.stringify({validation:{strictAvailable:true},presentation:{format:'ifccad',opaqueEntityCount:1},conversion:{preservation:{entries:[{result:'capturedTyped'}]}}});},convert_cad_to_ifccad(){return JSON.stringify({validation:{strictAvailable:true},presentation:{opaqueEntityCount:0}});}};
 const request={kind:'cad',drawingFormat:'ifccad',name:'a.dwg',preserveSplines:true,files:[{path:'a.dwg',bytes}]};
 const result=processBrowserRequest(request,wasm);assert.equal(selected.preserveSplines,true);assert.equal(result.presentation.opaqueEntityCount,1);
 assert.throws(()=>processBrowserRequest(request,{...wasm,conversion_capabilities(){return JSON.stringify({ifccad:{adjustableTolerance:true}});}}),/preservation|processor/i);
});

test('selected loss policy reaches the physical exporter and cannot fall back to Allow',()=>{
 let selected,legacyCalled=false;const wasm={open_ifccad(){return JSON.stringify({validation:{strictAvailable:true}});},export_ifccad(){legacyCalled=true;return '{}';},export_ifccad_with_options(...args){selected=JSON.parse(args.at(-1));return JSON.stringify({failure:{code:'CAD_EXCHANGE_LOSS_REJECTED'}});}};
 const request={kind:'ifccad',name:'widths.ifcx',files:[{path:'widths.ifcx',bytes}],exportConversionOptions:{lossPolicy:'reject'},export:{format:'dwg',version:'AC1032'}};
 const result=processBrowserRequest(request,wasm);assert.equal(selected.lossPolicy,'reject');assert.equal(result.failure.code,'CAD_EXCHANGE_LOSS_REJECTED');assert.equal(legacyCalled,false);
 assert.throws(()=>processBrowserRequest(request,{...wasm,export_ifccad_with_options:undefined}),/loss|reject|processor/i);assert.equal(legacyCalled,false);
 assert.throws(()=>processBrowserRequest({...request,exportConversionOptions:{lossPolicy:'guess'}},wasm),/loss policy/i);
});
