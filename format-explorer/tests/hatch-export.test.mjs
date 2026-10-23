import test from 'node:test';
import assert from 'node:assert/strict';
import {hatchExportState,showHatchExportRestrictions} from '../src/hatch-export.mjs';
import {cadVersions,cadOutputForSource} from '../src/cad-formats.mjs';
import {translate} from '../src/i18n.mjs';
test('selected format/version preflight follows source AC1024 without substitution',()=>{
 const source=cadOutputForSource('foundation.dwg','AC1024',{format:'dxf',version:'AC1032'});
 assert.deepEqual(source,{format:'dwg',version:'AC1024'});
 const issue={code:'PIXEL',message:'Pixel size requires DERIVED',policyEffect:'loss'};
 const presentation={hatchExportPreflight:cadVersions.flatMap(([version])=>['dxf','dwg'].map(format=>({format,version,restrictions:version==='AC1024'?[issue]:[]})))};
 for(const format of ['dxf','dwg'])for(const [version] of cadVersions){
  const allow=hatchExportState(presentation,{format,version});
  assert.equal(allow.blocked,false);assert.deepEqual(allow.restrictions,version==='AC1024'?[issue]:[]);
  const reject=hatchExportState(presentation,{format,version,lossPolicy:'reject'});
  assert.equal(reject.blocked,version==='AC1024');
 }
});
test('unsafe targets block independently of loss policy, replaced/stale sources do not retain old restrictions',()=>{
 const presentation={hatchExportPreflight:[{format:'dwg',version:'AC1015',restrictions:[{code:'GRADIENT',message:'Active gradients need DWG AC1018 or later',policyEffect:'block'}]}]};
 assert.equal(hatchExportState(presentation,{format:'dwg',version:'AC1015'}).blocked,true);
 assert.equal(hatchExportState(presentation,{format:'dxf',version:'AC1015'}).blocked,false);
 assert.equal(hatchExportState({}, {format:'dwg',version:'AC1015'}).restrictions.length,0);
 assert.equal(hatchExportState(presentation,{format:'dwg',version:'AC1015',stale:true}).blocked,true);
});
test('the visible disclosure names format/version and does not enable an unavailable download',()=>{
 const note={},button={disabled:false};const t=(key,args)=>`${key} ${JSON.stringify(args??{})}`;
 const presentation={hatchExportPreflight:[{format:'dwg',version:'AC1015',restrictions:[{code:'HATCH_TARGET_FEATURE_UNSUPPORTED',message:'gradient',policyEffect:'block'}]}]};
 showHatchExportRestrictions(note,button,presentation,{format:'dwg',version:'AC1015'},t);
 assert.equal(note.hidden,false);assert.match(note.textContent,/AC1015/);assert.match(note.textContent,/DWG/);assert.equal(button.disabled,true);
 showHatchExportRestrictions(note,button,{}, {format:'dxf',version:'AC1024',unavailable:true},t);
 assert.equal(note.hidden,true);assert.equal(button.disabled,true);
});
test('unavailable numeric preflight is a runtime-check notice, including under Reject',()=>{
 const presentation={hatchExportPreflight:[{format:'dxf',version:'AC1024',restrictions:[{code:'HATCH_PREFLIGHT_UNAVAILABLE',policyEffect:'check'}]}]};
 assert.equal(hatchExportState(presentation,{format:'dxf',version:'AC1024',lossPolicy:'reject'}).blocked,false);
});

test('repeated editing losses are summarized once while individual diagnostics and Reject remain intact',()=>{
 const restrictions=Array.from({length:10},(_,i)=>({code:'hatch-classic-settings-loss',location:`entity/${70+i}/classicSettings/definitionLines`,message:'CAD slots cannot retain the authored presence/value at this field; the complete editing record is omitted',policyEffect:'loss'}));
 const presentation={hatchExportPreflight:[{format:'dwg',version:'AC1024',restrictions}]};
 for(const language of ['nl','en']){
  const note={},button={},selection={format:'dwg',version:'AC1024'};
  const state=showHatchExportRestrictions(note,button,presentation,selection,(key,args)=>translate(language,key,args));
  assert.equal(note.textContent.split('AC1024').length-1,1,'one summary instead of ten full warnings');
  assert.match(note.textContent,/10/);
  assert.doesNotMatch(note.textContent,/CAD slots/,'known conversion loss is localized');
  assert.ok(note.textContent.length<400,'summary remains compact for a large drawing');
  assert.equal(button.disabled,false);
  assert.equal(state.restrictions,restrictions,'all located diagnostics remain available');
  showHatchExportRestrictions(note,button,presentation,{...selection,lossPolicy:'reject'},(key,args)=>translate(language,key,args));
  assert.equal(button.disabled,true);
 }
});

test('different unknown reasons and mandatory blocks are not hidden by warning grouping',()=>{
 const restrictions=[
  {code:'FUTURE',message:'First specific loss',policyEffect:'loss'},
  {code:'FUTURE',message:'Second specific loss',policyEffect:'loss'},
  {code:'HATCH_TARGET_FEATURE_UNSUPPORTED',policyEffect:'block'},
 ];
 const note={},button={};
 showHatchExportRestrictions(note,button,{hatchExportPreflight:[{format:'dwg',version:'AC1015',restrictions}]},{format:'dwg',version:'AC1015'},(key,args)=>translate('en',key,args));
 assert.match(note.textContent,/First specific loss/);assert.match(note.textContent,/Second specific loss/);
 assert.match(note.textContent,/AC1018/);assert.equal(button.disabled,true);
});

test('default recipe assessment stays advisory under Reject and is localized as a fresh output check',()=>{
 const presentation={hatchExportPreflight:[{format:'dwg',version:'AC1024',restrictions:[{code:'HATCH_RECIPE_PREFLIGHT_CHECK',policyEffect:'check'}]}]};
 for(const language of ['nl','en']){
  const note={},button={};
  showHatchExportRestrictions(note,button,presentation,{format:'dwg',version:'AC1024',lossPolicy:'reject'},(key,args)=>translate(language,key,args));
  assert.equal(button.disabled,false);
  assert.doesNotMatch(note.textContent,/cannot be retained|niet.*behouden/);
  assert.match(note.textContent,language==='nl'?/gekozen uitvoertolerantie/:/selected output tolerance/);
 }
});

test('line-pattern upright restrictions share target selection without a frontend version table',()=>{
 const presentation={linePatternExportPreflight:[{format:'dxf',version:'AC1024',restrictions:[{code:'TARGET_CODEC_LINE_PATTERN_UPRIGHT_LOSS',policyEffect:'loss',message:'upright becomes relative'}]}]};
 assert.equal(hatchExportState(presentation,{format:'dxf',version:'AC1024',lossPolicy:'reject'}).blocked,true);
 assert.equal(hatchExportState(presentation,{format:'dxf',version:'AC1024',lossPolicy:'allow'}).blocked,false);
 assert.deepEqual(hatchExportState(presentation,{format:'dwg',version:'AC1024'}).restrictions,[]);
});
