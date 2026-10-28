import {createFileClient,decodeBase64} from './browser-client.mjs';
import {initializeCadPreview} from './cad-preview.mjs';
import {initializeWorkspace} from './workspace.mjs';
import {conversionOptions} from './conversion-settings.mjs';
import {examples} from './examples.mjs';
import {ifccadEncoding} from './ifccad-encoding.mjs';
import {initializeToleranceSettings} from './tolerance-settings.mjs';
import {cadOutputForSource,supportsCadVersion} from './cad-formats.mjs';
import {hatchExportState,showHatchExportRestrictions} from './hatch-export.mjs';
import {nativeExportRequest} from './native-request.mjs';
const $=id=>document.getElementById(id),client=createFileClient();let source,controller,generation=0,stale=false,result,sourceCadVersion;
function showsCad(tab){return tab==='overview'||tab==='drawing';}
const workspace=initializeWorkspace({onTab:tab=>{if(showsCad(tab)&&!stale)preview.show();else preview.hide();toleranceSettings?.setView(tab);updateTolerance();},onSelection:key=>{if(!stale)void preview.selectElement(key);},onLayout:layout=>{if(!stale)void preview.selectLayout(layout);},onPreferences:()=>{preview.refresh?.();updatePreservation();updateTolerance();updateCadSourceNote();}}),t=(key,values)=>workspace.t(key,values),preview=initializeCadPreview({translate:t,onCadSelection:key=>{if(!stale)workspace.revealCadElement?.(key);},externalSettings:true});
const toleranceSettings=typeof initializeToleranceSettings==='function'?initializeToleranceSettings({translate:t,onApply:()=>$('apply-settings').click()}):null;
toleranceSettings?.setView(workspace.state?.tab||'overview');
function showStatus(message,quiet=false){workspace.setMessage?.(message,quiet);$("status").textContent=message;$("status").hidden=quiet;}
function drawingFormat(){return 'ifccad';}
function selectedOptions(stage='input'){return conversionOptions(toleranceSettings?toleranceSettings.selected(stage):{mode:$('tolerance-mode').value||'default',value:$('tolerance-value').value,unit:$('tolerance-unit').value||'mm'});}
function updatePreservation(){const cad=/\.(dxf|dwg)$/i.test($('file').files?.[0]?.name||'');const available=cad&&result?.conversionCapabilities?.ifccad?.splinePreservation===true;$('preservation-control').hidden=!available;$('preserve-splines').disabled=!available||!!controller;if(!available)$('preserve-splines').checked=false;}
function hatchSelection(){return {format:$('preview-format').value||'dxf',version:$('preview-version').value||'AC1032',lossPolicy:(source?.exportConversionOptions??source?.conversionOptions)?.lossPolicy||'allow',stale,unavailable:!result?.validation?.strictAvailable||!!controller};}
function updateCadSourceNote(){const label=t('downloadCadOutput',{format:($('preview-format').value||'dxf').toUpperCase(),version:$('preview-version').value||'AC1032'});$('cad-download').title=label;$('cad-download').ariaLabel=label;const unmatched=source?.kind==='cad'&&sourceCadVersion&&!supportsCadVersion(sourceCadVersion);$('cad-source-settings-note').hidden=!unmatched;$('cad-source-settings-note').textContent=unmatched?t('unsupportedSourceCadVersion',{sourceVersion:sourceCadVersion,outputVersion:$('preview-version').value}):'';showHatchExportRestrictions($('hatch-export-note'),$('cad-download'),result?.presentation,hatchSelection(),t);}
function updateTolerance(){const mode=$('tolerance-mode').value||'default';$('custom-tolerance').hidden=mode!=='custom';const unit=result?.presentation?.unit||result?.presentation?.graph?.data?.find(n=>n.attributes?.['ifccad::drawing'])?.attributes['ifccad::drawing'].lengthUnit,capable=result?.conversionCapabilities?.[drawingFormat()]?.adjustableTolerance;for(const option of Array.from($('tolerance-mode').options||[]))if(option.value!=='default')option.disabled=capable===false;if($('tolerance-mode').options?.[0])$('tolerance-mode').options[0].textContent=t(capable===false?'legacyTolerance':'defaultTolerance');$('tolerance-effective').textContent=(unit?t('modelUnit',{unit}):'');toleranceSettings?.setReport(result);$('tolerance-open').hidden=true;$('tolerance-fields').hidden=true;}
function finishBusy(){controller=null;$('partial-open').disabled=false;workspace.clearLoading();$('file').disabled=false;$('open').disabled=false;updatePreservation();updateCadSourceNote();}
function loadingInfo(name,kind,operation){return {name,action:operation?'exportingTo':kind==='cad'?'convertingTo':'openingFile',format:operation?.format?.toUpperCase()||('IFCCAD')};}
function clearSelection(){$('native-download-menu').hidePopover?.();generation++;controller?.abort();finishBusy();source=undefined;result=undefined;$('partial-open').hidden=true;$('partial-opening').hidden=true;sourceCadVersion=undefined;stale=false;preview.clear();workspace.clear();updateCadSourceNote();$('export').disabled=true;$('cad-download').disabled=true;$('drawing').hidden=true;}
function updatePartialOpening(value){
 const partial=value.partialOpening;
 $('partial-open').hidden=source?.kind!=='cad'||value.failure?.partialOpenAvailable!==true||source?.conversionOptions?.partialOpen===true;
 $('partial-opening').hidden=!partial;
 $('partial-opening-message').textContent=partial?t('partialNotice',{count:partial.omittedEntityCount,errors:partial.errorCount}):'';
 $('partial-opening-items').textContent=partial?(partial.entities||[]).map(e=>[e.handle,e.kind,e.ownerName,e.layer,e.reason].filter(Boolean).join(' · ')).join('\n'):'';
}
function show(value,operation){if(operation){if(['dxf','dwg'].includes(operation.format))workspace.setCadReport(value);$('export').disabled=!result?.validation?.strictAvailable||stale;$('cad-download').disabled=$('export').disabled;showStatus(value.failure?.message||t('ready'),!value.failure);return;}result=value;updatePartialOpening(value);const label='IFCCAD';$('content-title').textContent=label+'-'+t('contents').toLowerCase();$('export').title=t('downloadNative',{format:label});$('export').ariaLabel=$('export').title;$('drawing').hidden=!value.presentation&&!value.nativeSourceText;$('name').textContent=value.source?.name||source.name;
 workspace.show({...value,inspectedFormat:drawingFormat(),presentation:value.presentation?{...value.presentation,format:drawingFormat()}:null},value.nativeSourceText||'');
 $('export').disabled=!value.validation?.strictAvailable||stale;$('cad-download').disabled=$('export').disabled;$('preview-open').disabled=$('export').disabled&&source?.kind!=='cad';showStatus(value.failure?.message||t(value.validation?.strictAvailable?(value.presentation?.opaqueEntityCount>0?'readyOpaque':'ready'):'invalid'),!!value.validation?.strictAvailable&&!value.failure&&!(value.presentation?.opaqueEntityCount>0));updateTolerance();
}
function download(file){const url=URL.createObjectURL(new Blob([decodeBase64(file.base64)])),link=document.createElement('a');link.href=url;link.download=file.fileName||source.name;link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
async function run(operation,{newSource=false}={}){if(!source)return;$('partial-open').disabled=true;const current=++generation;controller=new AbortController();$('file').disabled=true;$('open').disabled=true;$('export').disabled=true;$('cad-download').disabled=true;
 workspace.setLoading({...loadingInfo(source.name,source.kind,operation),phase:'preparing'});
 try{const value=await client.open(operation?nativeExportRequest(source,result,operation):{...source},{signal:controller.signal,onProgress:phase=>{if(current===generation){workspace.setStatus(phase);workspace.setLoading({phase});}}});if(current!==generation)return;if(newSource&&source.kind==='cad'){sourceCadVersion=value.reader?.version;const output=cadOutputForSource(source.name,sourceCadVersion,{format:$('preview-format').value,version:$('preview-version').value});$('preview-format').value=output.format;$('preview-version').value=output.version;}stale=false;show(value,operation);updateCadSourceNote();
  if(!operation){if(newSource)preview.setSource({...source},!!value.validation?.strictAvailable&&!value.failure,value);else preview.updateSource({...source},!!value.validation?.strictAvailable&&!value.failure,value);if(value.validation?.strictAvailable&&!value.failure)void preview.select({format:$('preview-format').value||'dxf',version:$('preview-version').value||'AC1032'});}
  if(operation&&value.export?.download&&!value.failure)download(value.export.download);
 }catch(e){if(current===generation){showStatus(e.name==='AbortError'?t('cancelled'):e.message);if(operation&&e.name!=='AbortError')show({failure:{message:e.message}},operation);if(!operation&&e.name!=='AbortError')preview.setSource({...source},false);}}
 finally{if(current===generation)finishBusy();}
}
async function openFile(file){clearSelection();const readingGeneration=generation;if(!file){workspace.setStatus('chooseFirst');return;}if(file.size>64*1024*1024){workspace.setStatus('tooLarge');return;}const kind=/\.(dxf|dwg)$/i.test(file.name)?'cad':ifccadEncoding(file.name)?'ifccad':'drawing';$('file').disabled=true;$('open').disabled=true;workspace.setStatus('loading');workspace.setLoading({...loadingInfo(file.name,kind),phase:'loading'});
 try{toleranceSettings?.apply();const options=selectedOptions('input'),exportOptions=selectedOptions('output'),bytes=await file.arrayBuffer();if(readingGeneration!==generation)return;source={kind,name:file.name,...(kind==='cad'?{drawingFormat:'ifccad',preserveSplines:result?.conversionCapabilities?.ifccad?.splinePreservation===true&&$('preserve-splines').checked===true}:{}),conversionOptions:options,exportConversionOptions:exportOptions,files:[{path:file.name,bytes}]};}
 catch(e){if(readingGeneration===generation){showStatus(t('readFailed')+' '+e.message);finishBusy();}return;}
 const openedGeneration=generation+1;await run(undefined,{newSource:true});if(generation!==openedGeneration)return false;if(showsCad(workspace.state?.tab)&&(result?.validation?.strictAvailable||source?.kind==='cad'))void preview.show();return true;
}
$('partial-open').onclick=async()=>{if(!source||!result?.failure?.partialOpenAvailable||controller)return;$('partial-open').hidden=true;source={...source,conversionOptions:{...source.conversionOptions,partialOpen:true}};await run();};
$('partial-opening-report').onclick=()=>{if(!result?.partialOpening)return;const url=URL.createObjectURL(new Blob([JSON.stringify({source:result.source,partialOpening:result.partialOpening,diagnostics:result.conversion?.diagnostics||[]},null,2)],{type:'application/json'})),link=document.createElement('a');link.href=url;link.download=source.name+'.omissions.json';link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);};
$('file').onchange=()=>{clearSelection();if(/\.(dxf|dwg)$/i.test($('file').files?.[0]?.name||''))$('preserve-splines').checked=false;updatePreservation();};$('open').onclick=()=>openFile($('file').files[0]);
function settingsChanged(){$('native-download-menu').hidePopover?.();updatePreservation();stale=!!source;if(stale){preview.cancel();workspace.setStatus('stale');$('export').disabled=true;$('cad-download').disabled=true;$('preview-open').disabled=true;}updatePreservation();updateTolerance();updateCadSourceNote();}
for(const name of ['preview-format','preview-version','preserve-splines'])$(name).onchange=settingsChanged;
$('apply-settings').onclick=async()=>{
 if(!source)return;
 // Output choices do not rerun input conversion or replace the native snapshot.
 stale=false;workspace.setCadReport(null);workspace.setStatus('ready');$('export').disabled=!result?.validation?.strictAvailable;$('cad-download').disabled=$('export').disabled;
 await preview.select({format:$('preview-format').value,version:$('preview-version').value});
 updateCadSourceNote();
 if(showsCad(workspace.state?.tab)&&result?.validation?.strictAvailable)void preview.show();
};
$('export').onclick=()=>{
 if(stale||$('export').disabled)return;
 const menu=$('native-download-menu'),rect=$('export').getBoundingClientRect();
 menu.style.left=Math.max(8,Math.min(rect.left,innerWidth-344))+'px';
 menu.style.top=Math.max(8,Math.min(rect.bottom+8,innerHeight-360))+'px';
 menu.showPopover();
};
for(const encoding of ['json','cbor','cbor-zstd'])$('native-'+encoding).onclick=()=>{
 $('native-download-menu').hidePopover();
 if(!stale&&!$('export').disabled)return run({format:drawingFormat(),...(encoding==='json'?{}:{encoding})});
};$('cad-download').onclick=()=>stale||hatchExportState(result?.presentation,hatchSelection()).blocked?undefined:run({format:$('preview-format').value,version:$('preview-version').value});
function cancel(){generation++;controller?.abort();finishBusy();workspace.setStatus('cancelled');}
$('loading-cancel').onclick=cancel;
$('preview-open').onclick=()=>{if(!stale)workspace.selectTab('drawing');};
for(const example of examples){const option=document.createElement('option');option.value=example.path;option.textContent=example.name;$('examples').append(option);}
async function openExample(path){const example=examples.find(x=>x.path===path);if(!example)return;clearSelection();const loadingGeneration=generation;controller=new AbortController();$('file').disabled=true;$('open').disabled=true;workspace.setLoading({...loadingInfo(example.path.split('/').at(-1),example.format==='IFCCAD'?'ifccad':'drawing'),phase:'loading'});try{const response=await fetch(example.path,{signal:controller.signal});if(!response.ok)throw Error('Example unavailable');const bytes=await response.arrayBuffer();if(loadingGeneration!==generation)return;$('file').value='';if(await openFile({name:example.path.split('/').at(-1),size:bytes.byteLength,arrayBuffer:async()=>bytes}))$('examples').value=example.path;}catch(e){if(loadingGeneration===generation){clearSelection();showStatus(e.message);}}}
$('examples').onchange=()=>openExample($('examples').value);
$('preserve-splines').checked=false;updatePreservation();updateTolerance();updateCadSourceNote();
if(examples.length)openExample(examples[0].path);
