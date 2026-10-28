import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import vm from 'node:vm';
import {ifccadEncoding} from '../src/ifccad-encoding.mjs';
import * as cadFormats from '../src/cad-formats.mjs';
import * as hatchExport from '../src/hatch-export.mjs';

test('native preview readiness never claims that CAD output was prepared or read back',async()=>{
 const elements=new Map(),statuses=[],reports=[];let hooks;
 const element=id=>{if(!elements.has(id))elements.set(id,{disabled:false,hidden:true,value:'',hidePopover(){}});return elements.get(id);};
 const code=(await readFile(new URL('../src/app.mjs',import.meta.url),'utf8')).replace(/^import .*;\r?\n/gm,'');
 vm.runInNewContext(code,{document:{getElementById:element},examples:[],ifccadEncoding,...cadFormats,...hatchExport,AbortController,
  conversionOptions:()=>({tolerance:{mode:'default'}}),createFileClient:()=>({}),
  initializeWorkspace:()=>({t:key=>key,setCadStatus:value=>statuses.push(value),setCadReport:value=>reports.push(value)}),
  initializeCadPreview:options=>{hooks=options;return {};},
 });
 hooks.onState?.({phase:'ready',format:'dxf',version:'AC1032'});
 hooks.onReport?.({failure:{message:'Host failed'}});
 assert.deepEqual(statuses,[]);assert.deepEqual(reports,[]);
});
