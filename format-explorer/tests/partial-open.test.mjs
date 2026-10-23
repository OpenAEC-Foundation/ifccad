import * as hatchExport from '../src/hatch-export.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import vm from 'node:vm';
import {translate} from '../src/i18n.mjs';
import {ifccadEncoding} from '../src/ifccad-encoding.mjs';
import * as cadFormats from '../src/cad-formats.mjs';
import {processBrowserRequest} from '../src/browser-worker.mjs';

test('partial opening is an explicit retry after an eligible error, persists through export and resets on new input',async()=>{
 const elements=new Map(),requests=[],reports=[];let recoverable=true;
 const element=id=>{if(!elements.has(id))elements.set(id,{hidden:true,disabled:false,value:'',textContent:'',checked:false,hidePopover(){}});return elements.get(id);};
 const workspace={t:(key,v)=>translate('nl',key,v),show(){},clear(){},setMessage(){},setStatus(){},setLoading(){},clearLoading(){},setCadReport(){}};
 const code=(await readFile(new URL('../src/app.mjs',import.meta.url),'utf8')).replace(/^import .*;\r?\n/gm,'');
 const result={validation:{strictAvailable:true},conversion:{diagnostics:[{code:'source-field',location:'field',message:'not represented'}]},partialOpening:{errorCount:1,omittedEntityCount:2,entities:[{handle:'0xA',kind:'HATCH',layer:'wall',reason:'GapExceeded'},{handle:'0xB',kind:'TEXT',reason:'unsupported'}],errors:[{code:'HatchPreparation/GapExceeded'}]}};
 vm.runInNewContext(code,{initializeWorkspace:()=>workspace,conversionOptions:()=>({tolerance:{mode:'default'}}),examples:[],ifccadEncoding,...cadFormats,...hatchExport,Blob,URL:{createObjectURL(blob){reports.push(blob);return 'blob:report';},revokeObjectURL(){}},setTimeout(){},document:{getElementById:element,createElement:()=>({click(){}})},createFileClient:()=>({async open(r){requests.push(r);return r.conversionOptions.partialOpen?result:{validation:{strictAvailable:false},failure:{code:'IFCCAD_CONVERSION_FAILED',message:'Gap exceeds limit',partialOpenAvailable:recoverable}};}}),initializeCadPreview:()=>({clear(){},setSource(){},updateSource(){},cancel(){},select(){return Promise.resolve();}}),AbortController});
 assert.equal(element('partial-open').hidden,true);
 element('file').files=[{name:'bad.dxf',size:1,arrayBuffer:async()=>new ArrayBuffer(1)}];
 await element('open').onclick();
 assert.equal(requests.length,1);assert.equal(requests[0].conversionOptions.partialOpen,undefined);
 assert.equal(element('partial-open').hidden,false);assert.match(element('status').textContent,/Gap exceeds/);
 assert.equal(element('status').hidden,false);
 assert.equal([...elements.values()].filter(item=>!item.hidden&&item.textContent==='Gap exceeds limit').length,1,'The opening error appears once, beside its recovery action');
 await element('partial-open').onclick();
 assert.equal(requests[1].conversionOptions.partialOpen,true);assert.equal(element('export').disabled,false);
 assert.equal(element('partial-open').hidden,true);assert.equal(element('partial-opening').hidden,false);
 assert.match(element('partial-opening-message').textContent,/2/);assert.match(element('partial-opening-items').textContent,/0xA.*HATCH/);
 element('partial-opening-report').onclick();const saved=JSON.parse(await reports[0].text());assert.deepEqual(saved.partialOpening,result.partialOpening);assert.deepEqual(saved.diagnostics,result.conversion.diagnostics);
 await element('cad-download').onclick();assert.equal(requests.at(-1).conversionOptions.partialOpen,true);
 assert.equal(element('partial-opening').hidden,false);
 await element('open').onclick();assert.equal(requests.at(-1).conversionOptions.partialOpen,undefined);
 assert.equal(element('partial-opening').hidden,true);
 recoverable=false;await element('open').onclick();assert.equal(element('partial-open').hidden,true);
});

test('worker forwards partial import on every CAD pass and retains omission metadata for CAD export',()=>{
 const calls=[];const report={errorCount:1,omittedEntityCount:1,entities:[{handle:'0xA'}],errors:[]};
 const wasm={conversion_capabilities:()=>JSON.stringify({ifccad:{coordinateToleranceDefault:true,partialOpening:true}}),convert_cad_to_ifccad_with_options(...args){calls.push(JSON.parse(args.at(-1)));return JSON.stringify({validation:{strictAvailable:true},partialOpening:report,export:{download:{base64:'e30='}}});},export_ifccad_with_options(){return JSON.stringify({validation:{strictAvailable:true},export:{download:{base64:'AA=='}}});}};
 const request={kind:'cad',name:'bad.dxf',files:[{path:'bad.dxf',bytes:new ArrayBuffer(1)}],conversionOptions:{partialOpen:true},export:{format:'dxf',version:'AC1032'}};
 const out=processBrowserRequest(request,wasm);assert.equal(calls[0].partialOpen,true);assert.deepEqual(out.partialOpening,report);
 const old={...wasm,conversion_capabilities:()=>JSON.stringify({ifccad:{coordinateToleranceDefault:true}})};
 assert.throws(()=>processBrowserRequest(request,old),/partial/i);
});
