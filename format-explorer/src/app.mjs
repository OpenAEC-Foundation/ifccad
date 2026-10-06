import {createFileClient,decodeBase64} from './browser-client.mjs';
import {initializeCadPreview} from './cad-preview.mjs';
const $=id=>document.getElementById(id),client=createFileClient(),preview=initializeCadPreview();let source,controller;
function drawingFormat(){return source?.kind==='ifccad'?'ifccad':source?.drawingFormat||'ocdraw';}
function updateInputControls(){
 const format=$('file').files?.[0]?.name.match(/\.(dwg|dxf)$/i)?.[1]?.toUpperCase();
 $('drawing-format-control').hidden=!format;$('drawing-format-label').textContent=format?format+' omzetten naar':'';$('drawing-format').disabled=!format||!!controller;
 const preservationAvailable=!!format&&$('drawing-format').value!=='ifccad';$('preservation-control').hidden=!preservationAvailable;$('preserve-splines').disabled=!preservationAvailable||!!controller;
}
function clearSelection(){source=undefined;preview.clear();$('export').disabled=true;$('cad-download').disabled=true;$('drawing').hidden=true;}
function show(result){
 const label=drawingFormat()==='ifccad'?'IFCCAD':'OCDraw';$('content-title').textContent=label+'-inhoud';$('export').textContent=label+' downloaden';
 $('drawing').hidden=false;$('name').textContent=result.source?.name||source.name;
 $('report').textContent=JSON.stringify({reader:result.reader,conversion:result.conversion,validation:result.validation,failure:result.failure,export:result.export?{format:result.export.format,diagnostics:result.export.diagnostics,fileCheck:result.export.fileCheck}:undefined},null,2);
 $('records').textContent=JSON.stringify(result.presentation,null,2)||'Geen gevalideerde tekeninginformatie.';
 $('export').disabled=!result.validation?.strictAvailable||!!result.failure;$('cad-download').disabled=$('export').disabled;
 $('status').textContent=result.failure?.message|| (result.validation?.strictAvailable?(result.presentation?.opaqueEntityCount>0?'Tekening gecontroleerd. Sommige vormen zijn alleen als brongegevens opgeslagen.':'Tekening gecontroleerd.'):'De tekening kon niet worden gevalideerd.');
}
async function run(operation){
 if(!source)return;controller=new AbortController();$('file').disabled=true;$('open').disabled=true;$('cancel').disabled=false;$('export').disabled=true;$('cad-download').disabled=true;$('drawing-format').disabled=true;
 $('preserve-splines').disabled=true;
 try{const result=await client.open({...source,...(operation?{export:operation}:{})},{signal:controller.signal,onProgress:phase=>$('status').textContent=phase});show(result);if(!operation)preview.setSource({...source},!!result.validation?.strictAvailable&&!result.failure);
  if(operation&&result.export?.download&&!result.failure){const file=result.export.download,url=URL.createObjectURL(new Blob([decodeBase64(file.base64)])),link=document.createElement('a');link.href=url;link.download=file.fileName||source.name+'.'+operation.format;link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
 }catch(e){$('status').textContent=e.name==='AbortError'?'Geannuleerd.':e.message;if(!operation&&e.name!=='AbortError')preview.setSource({...source},false);}finally{controller=null;$('file').disabled=false;$('open').disabled=false;$('cancel').disabled=true;updateInputControls();}
}
$('file').onchange=()=>{clearSelection();updateInputControls();};
$('open').onclick=async()=>{
 clearSelection();updateInputControls();
 const file=$('file').files[0];if(!file){$('status').textContent='Kies eerst een bestand.';return;}if(file.size>64*1024*1024){$('status').textContent='Dit bestand is groter dan 64 MiB.';return;}
 $('file').disabled=true;$('open').disabled=true;$('status').textContent='Bestand lezen…';
 try{const kind=/\.(dxf|dwg)$/i.test(file.name)?'cad':/\.ifcx(?:\.json)?$/i.test(file.name)?'ifccad':'drawing';if(kind!=='cad')$('drawing-format').value=kind==='ifccad'?'ifccad':'ocdraw';source={kind,name:file.name,...(kind==='cad'?{drawingFormat:$('drawing-format').value||'ocdraw',preserveSplines:$('preserve-splines').checked===true&&$('drawing-format').value!=='ifccad'}:{}),files:[{path:file.name,bytes:await file.arrayBuffer()}]};}
 catch(e){$('status').textContent='Het bestand kon niet worden gelezen. Sluit het eventueel in het andere programma en kies het opnieuw. '+e.message;return;}
 finally{$('file').disabled=false;$('open').disabled=false;}
 await run();
};
$('drawing-format').onchange=async()=>{updateInputControls();if(source?.kind!=='cad')return;source={...source,drawingFormat:$('drawing-format').value,preserveSplines:$('preserve-splines').checked===true&&$('drawing-format').value!=='ifccad'};preview.clear();await run();};
$('preserve-splines').onchange=async()=>{if(source?.kind!=='cad'||drawingFormat()!=='ocdraw')return;source={...source,preserveSplines:$('preserve-splines').checked===true};preview.clear();await run();};
$('export').onclick=()=>run({format:drawingFormat()});$('cad-download').onclick=()=>run({format:$('preview-format').value,version:$('preview-version').value});$('cancel').onclick=()=>controller?.abort();
updateInputControls();
