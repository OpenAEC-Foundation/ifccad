import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {createCadPreviewController} from '../src/cad-preview.mjs';

const fixture={ifcx:{data:[{type:'openaec:Drawing',path:'drawing-main',attributes:{name:'Model'}}]}};
const download=(format='dxf',version='AC1032')=>({failure:null,export:{format,effectiveVersion:version,download:{format,base64:'YQ==',byteLength:1}}});
const tick=()=>new Promise(resolve=>setTimeout(resolve,0));

test('CAD import opens original and 2018 roundtrip in one viewer session',async()=>{
 const calls=[],requests=[];
 const controller=createCadPreviewController({openExport:async request=>{requests.push(request);return download('dwg');},openSession:async()=>({openOriginal:async(...args)=>calls.push(['original',...args]),replaceGenerated:async(...args)=>calls.push(['generated',...args]),close(){}})});
 controller.setSource({kind:'cad',name:'plan.dwg',files:[{path:'plan.dwg',base64:'Yg=='}]},fixture,{name:'plan.dwg',base64:'Yg=='});
 await controller.show();
 assert.equal(requests[0].export.version,'AC1032');assert.equal(requests[0].export.format,'dwg');
 assert.deepEqual(calls.map(c=>c[0]),['original','generated']);
 assert.match(calls[1][2],/2018/);
});

test('IFCCAD package with no CAD source opens only generated DXF',async()=>{
 const calls=[];
 const controller=createCadPreviewController({openExport:async()=>download(),openSession:async()=>({openOriginal:async()=>calls.push('original'),replaceGenerated:async()=>calls.push('generated'),close(){}})});
 controller.setSource({kind:'package',name:'sample',files:[]},fixture,null);
 await controller.show();assert.deepEqual(calls,['generated']);assert.equal(controller.state.version,'AC1032');assert.equal(controller.state.format,'dxf');
});

test('changing version invalidates a stale export result',async()=>{
 const pending=[],shown=[];
 const controller=createCadPreviewController({openExport:()=>new Promise(resolve=>pending.push(resolve)),openSession:async()=>({openOriginal:async()=>{},replaceGenerated:async(base64,name)=>shown.push(name),close(){}})});
 controller.setSource({kind:'package',name:'sample',files:[]},fixture,null);
 const initial=controller.show();await tick();
 const changed=controller.select({version:'AC1027'});await tick();
 pending[1](download('dxf','AC1027'));await changed;pending[0](download());await initial;
 assert.deepEqual(shown,['via-ifccad-2013.dxf']);
});

test('failed export leaves the original open and exposes diagnostics',async()=>{
 const calls=[];
 const failed={failure:{stage:'exporting',code:'CAD_WRITE_FAILED',message:'bad entity'},export:{format:'dxf',download:null}};
 const controller=createCadPreviewController({openExport:async()=>failed,openSession:async()=>({openOriginal:async()=>calls.push('original'),replaceGenerated:async()=>calls.push('generated'),close(){}})});
 controller.setSource({kind:'cad',name:'plan.dxf',files:[]},fixture,{name:'plan.dxf',base64:'YQ=='});
 await controller.show();assert.deepEqual(calls,['original']);assert.equal(controller.state.result,failed);
});

test('returning to the same preview reuses its generated file and document tab',async()=>{
 let exports=0,opens=0;
 const controller=createCadPreviewController({openExport:async()=>{exports++;return download();},openSession:async()=>({openOriginal:async()=>{},replaceGenerated:async()=>{opens++;},close(){}})});
 controller.setSource({kind:'package',name:'sample',files:[]},fixture,null);
 await controller.show();controller.hide();await controller.show();
 assert.equal(exports,1);assert.equal(opens,1);
});

test('a viewer finishing after a source change is discarded',async()=>{
 let finishOld,oldClosed=false;const generated=[];
 const old=new Promise(resolve=>finishOld=resolve);
 let sessions=0;
 const controller=createCadPreviewController({openExport:async()=>download(),openSession:async()=>{
  sessions++;if(sessions===1)return old;
  return {openOriginal:async()=>{},replaceGenerated:async()=>generated.push('new'),close(){}};
 }});
 controller.setSource({kind:'package',name:'first',files:[]},fixture,null);
 const first=controller.show();await tick();
 controller.setSource({kind:'package',name:'second',files:[]},fixture,null);
 const second=controller.show();finishOld({openOriginal:async()=>{},replaceGenerated:async()=>generated.push('old'),close(){oldClosed=true;}});
 await Promise.all([first,second]);
 assert.equal(oldClosed,true);assert.deepEqual(generated,['new']);
});

test('preview controls expose drawing, format, version and a return to structure',async()=>{
 const html=await readFile(new URL('../src/index.html',import.meta.url),'utf8');
 for(const id of ['preview-open','preview-close','preview-drawing','preview-format','preview-version','preview-frame','preview-report','preview-download'])assert.match(html,new RegExp(`id="${id}"`));
});
